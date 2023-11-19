use std::{convert::TryInto, marker::PhantomData};

pub mod column;
pub use column::*;

pub mod styling;
pub use styling::*;

pub mod props;
pub use props::DataTableProps;

pub mod state;
pub use state::{DataTableState, ScrollDirection};

pub mod draw;
pub use draw::*;

pub mod data_type;
pub use data_type::*;

pub mod sortable;
pub use sortable::*;

use crate::utils::gen_util::ClampExt;

/// A [`DataTable`] is a component that displays data in a tabular form.
///
/// Note that [`DataTable`] takes a generic type `S`, bounded by [`SortType`]. This controls whether this table
/// expects sorted data or not, with two expected types:
///
/// - [`Unsortable`]: The default if otherwise not specified. This table does not expect sorted data.
/// - [`Sortable`]: This table expects sorted data, and there are helper functions to
///   facilitate things like sorting based on a selected column, shortcut column selection support, mouse column
///   selection support, etc.
pub struct DataTable<DataType, Header, S = Unsortable, C = Column<Header>> {
    pub columns: Vec<C>,
    pub state: DataTableState,
    pub props: DataTableProps,
    pub styling: DataTableStyling,
    data: Vec<DataType>,
    sort_type: S,
    first_draw: bool,
    _pd: PhantomData<(DataType, S, Header)>,
}

impl<DataType: DataToCell<H>, H: ColumnHeader> DataTable<DataType, H, Unsortable, Column<H>> {
    pub fn new<C: Into<Vec<Column<H>>>>(
        columns: C, props: DataTableProps, styling: DataTableStyling,
    ) -> Self {
        Self {
            columns: columns.into(),
            state: DataTableState::default(),
            props,
            styling,
            data: vec![],
            sort_type: Unsortable,
            first_draw: true,
            _pd: PhantomData,
        }
    }
}

impl<DataType: DataToCell<H>, H: ColumnHeader, S: SortType, C: DataTableColumn<H>>
    DataTable<DataType, H, S, C>
{
    /// Sets the scroll position to the first value.
    pub fn set_first(&mut self) {
        self.state.current_index = 0;
        self.state.scroll_direction = ScrollDirection::Up;
    }

    /// Sets the scroll position to the last value.
    pub fn set_last(&mut self) {
        self.state.current_index = self.data.len().saturating_sub(1);
        self.state.scroll_direction = ScrollDirection::Down;
    }

    /// Updates the scroll position to be valid for the number of entries.
    pub fn set_data(&mut self, data: Vec<DataType>) {
        self.data = data;
        let max_pos = self.data.len().saturating_sub(1);
        if self.state.current_index > max_pos {
            self.state.current_index = max_pos;
            self.state.display_start_index = 0;
            self.state.scroll_direction = ScrollDirection::Down;
        }
    }

    /// Increments the scroll position if possible by a positive/negative offset. If there is a
    /// valid change, this function will also return the new position wrapped in an [`Option`].
    pub fn increment_position(&mut self, change: i64) -> Option<usize> {
        let max_index = self.data.len();
        let current_index = self.state.current_index;

        if change == 0
            || (change > 0 && current_index == max_index)
            || (change < 0 && current_index == 0)
        {
            return None;
        }

        let csp: Result<i64, _> = self.state.current_index.try_into();
        if let Ok(csp) = csp {
            let proposed: Result<usize, _> = (csp + change).try_into();
            if let Ok(proposed) = proposed {
                if proposed < self.data.len() {
                    self.state.current_index = proposed;
                    self.state.scroll_direction = if change < 0 {
                        ScrollDirection::Up
                    } else {
                        ScrollDirection::Down
                    };

                    return Some(self.state.current_index);
                }
            }
        }

        None
    }

    /// Updates the scroll position to a selected index.
    #[allow(clippy::comparison_chain)]
    pub fn set_position(&mut self, new_index: usize) {
        let new_index = new_index.clamp_upper(self.data.len().saturating_sub(1));
        if self.state.current_index < new_index {
            self.state.scroll_direction = ScrollDirection::Down;
        } else if self.state.current_index > new_index {
            self.state.scroll_direction = ScrollDirection::Up;
        }
        self.state.current_index = new_index;
    }

    /// Returns the current scroll index.
    pub fn current_index(&self) -> usize {
        self.state.current_index
    }

    /// Optionally returns the currently selected item, if there is one.
    pub fn current_item(&self) -> Option<&DataType> {
        self.data.get(self.state.current_index)
    }

    /// Returns tui-rs' internal selection.
    pub fn tui_selected(&self) -> Option<usize> {
        self.state.table_state.selected()
    }
}

#[cfg(test)]
mod test {
    use super::*;

    #[derive(Clone, PartialEq, Eq, Debug)]
    struct TestType {
        index: usize,
    }

    impl DataToCell<&'static str> for TestType {
        fn to_cell(
            &self, _column: &&'static str, _calculated_width: u16,
        ) -> Option<tui::text::Text<'_>> {
            None
        }

        fn column_widths<C: DataTableColumn<&'static str>>(
            _data: &[Self], _columns: &[C],
        ) -> Vec<u16>
        where
            Self: Sized,
        {
            vec![]
        }
    }

    #[test]
    fn test_data_table_operations() {
        let columns = [Column::hard("a", 10), Column::hard("b", 10)];
        let props = DataTableProps {
            title: Some("test".into()),
            table_gap: 1,
            left_to_right: false,
            is_basic: false,
            show_table_scroll_position: true,
            show_current_entry_when_unfocused: false,
        };
        let styling = DataTableStyling::default();

        let mut table = DataTable::new(columns, props, styling);
        table.set_data((0..=4).map(|index| TestType { index }).collect::<Vec<_>>());

        table.set_last();
        assert_eq!(table.current_index(), 4);
        assert_eq!(table.state.scroll_direction, ScrollDirection::Down);

        table.set_first();
        assert_eq!(table.current_index(), 0);
        assert_eq!(table.state.scroll_direction, ScrollDirection::Up);

        table.set_position(4);
        assert_eq!(table.current_index(), 4);
        assert_eq!(table.state.scroll_direction, ScrollDirection::Down);

        table.set_position(100);
        assert_eq!(table.current_index(), 4);
        assert_eq!(table.state.scroll_direction, ScrollDirection::Down);
        assert_eq!(table.current_item(), Some(&TestType { index: 4 }));

        table.increment_position(-1);
        assert_eq!(table.current_index(), 3);
        assert_eq!(table.state.scroll_direction, ScrollDirection::Up);
        assert_eq!(table.current_item(), Some(&TestType { index: 3 }));

        table.increment_position(-3);
        assert_eq!(table.current_index(), 0);
        assert_eq!(table.state.scroll_direction, ScrollDirection::Up);
        assert_eq!(table.current_item(), Some(&TestType { index: 0 }));

        table.increment_position(-3);
        assert_eq!(table.current_index(), 0);
        assert_eq!(table.state.scroll_direction, ScrollDirection::Up);
        assert_eq!(table.current_item(), Some(&TestType { index: 0 }));

        table.increment_position(1);
        assert_eq!(table.current_index(), 1);
        assert_eq!(table.state.scroll_direction, ScrollDirection::Down);
        assert_eq!(table.current_item(), Some(&TestType { index: 1 }));

        table.increment_position(3);
        assert_eq!(table.current_index(), 4);
        assert_eq!(table.state.scroll_direction, ScrollDirection::Down);
        assert_eq!(table.current_item(), Some(&TestType { index: 4 }));

        table.increment_position(10);
        assert_eq!(table.current_index(), 4);
        assert_eq!(table.state.scroll_direction, ScrollDirection::Down);
        assert_eq!(table.current_item(), Some(&TestType { index: 4 }));

        table.set_data((0..=2).map(|index| TestType { index }).collect::<Vec<_>>());
        assert_eq!(table.current_index(), 2);
        assert_eq!(table.state.scroll_direction, ScrollDirection::Down);
        assert_eq!(table.current_item(), Some(&TestType { index: 2 }));
    }
}
