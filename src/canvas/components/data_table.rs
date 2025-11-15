pub mod column;
pub mod data_type;
pub mod draw;
pub mod props;
pub mod sortable;
pub mod state;
pub mod styling;

use std::{convert::TryInto, marker::PhantomData};

pub use column::*;
pub use data_type::*;
pub use draw::*;
pub use props::DataTableProps;
pub use sortable::*;
pub use state::{DataTableState, ScrollDirection};
pub use styling::*;

use crate::utils::general::ClampExt;

/// A [`DataTable`] is a component that displays data in a tabular form.
///
/// Note that [`DataTable`] takes a generic type `S`, bounded by [`SortType`].
/// This controls whether this table expects sorted data or not, with two
/// expected types:
///
/// - [`Unsortable`]: The default if otherwise not specified. This table does
///   not expect sorted data.
/// - [`Sortable`]: This table expects sorted data, and there are helper
///   functions to facilitate things like sorting based on a selected column,
///   shortcut column selection support, mouse column selection support, etc.
///
/// FIXME: We already do all the text width checks - can we skip the underlying ones?
pub struct DataTable<DataType, Header, S = Unsortable, C = Column<Header>> {
    pub columns: Vec<C>,
    pub state: DataTableState,
    pub props: DataTableProps,
    pub styling: DataTableStyling,
    data: Vec<DataType>,
    sort_type: S,
    first_draw: bool,
    first_index: Option<usize>,
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
            first_index: None,
            _pd: PhantomData,
        }
    }
}

impl<DataType: DataToCell<H>, H: ColumnHeader, S: SortType, C: DataTableColumn<H>>
    DataTable<DataType, H, S, C>
{
    /// Sets the default value selected on first initialization, if possible.
    pub fn first_draw_index(mut self, first_index: usize) -> Self {
        self.first_index = Some(first_index);
        self
    }

    /// Sets the scroll position to the first value.
    pub fn scroll_to_first(&mut self) {
        self.state.current_index = 0;
        self.state.scroll_direction = ScrollDirection::Up;
    }

    /// Sets the scroll position to the last value.
    pub fn scroll_to_last(&mut self) {
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

    /// Increments the scroll position if possible by a positive/negative
    /// offset. If there is a valid change, this function will also return
    /// the new position wrapped in an [`Option`].
    ///
    /// Note that despite the name, this handles both incrementing (positive
    /// change) and decrementing (negative change).
    pub fn increment_position(&mut self, change: i64) -> Option<usize> {
        let num_entries = self.data.len();

        if num_entries == 0 {
            return None;
        }

        let Ok(current_index): Result<i64, _> = self.state.current_index.try_into() else {
            return None;
        };

        // We do this to clamp the proposed index to 0 if the change is greater
        // than the number of entries left from the current index. This gives
        // a more intuitive behaviour when using things like page up/down.
        let proposed = current_index + change;

        // We check num_entries > 0 above.
        self.state.current_index = proposed.clamp(0, (num_entries - 1) as i64) as usize;

        self.state.scroll_direction = if change < 0 {
            ScrollDirection::Up
        } else {
            ScrollDirection::Down
        };

        Some(self.state.current_index)
    }

    /// Updates the scroll position to a selected index.
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

    /// Returns ratatui's internal selection.
    pub fn ratatui_selected(&self) -> Option<usize> {
        self.state.table_state.selected()
    }
}

#[cfg(test)]
mod test {
    use std::{borrow::Cow, num::NonZeroU16};

    use super::*;

    #[derive(Clone, PartialEq, Eq, Debug)]
    struct TestType {
        index: usize,
    }

    impl DataToCell<&'static str> for TestType {
        fn to_cell_text(
            &self, _column: &&'static str, _calculated_width: NonZeroU16,
        ) -> Option<Cow<'static, str>> {
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

    fn create_test_table() -> DataTable<TestType, &'static str> {
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

        DataTable::new(columns, props, styling)
    }

    #[test]
    fn test_scrolling() {
        let mut table = create_test_table();
        table.set_data((0..=4).map(|index| TestType { index }).collect::<Vec<_>>());

        table.scroll_to_last();
        assert_eq!(table.current_index(), 4);
        assert_eq!(table.state.scroll_direction, ScrollDirection::Down);

        table.scroll_to_first();
        assert_eq!(table.current_index(), 0);
        assert_eq!(table.state.scroll_direction, ScrollDirection::Up);
    }

    #[test]
    fn test_set_position() {
        let mut table = create_test_table();
        table.set_data((0..=4).map(|index| TestType { index }).collect::<Vec<_>>());

        table.set_position(4);
        assert_eq!(table.current_index(), 4);
        assert_eq!(table.state.scroll_direction, ScrollDirection::Down);

        table.set_position(100);
        assert_eq!(table.current_index(), 4);
        assert_eq!(table.state.scroll_direction, ScrollDirection::Down);
        assert_eq!(table.current_item(), Some(&TestType { index: 4 }));
    }

    #[test]
    fn test_increment_position() {
        let mut table = create_test_table();
        table.set_data((0..=4).map(|index| TestType { index }).collect::<Vec<_>>());

        table.set_position(4);
        assert_eq!(table.current_index(), 4);
        assert_eq!(table.state.scroll_direction, ScrollDirection::Down);

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

        // Make sure that overscrolling up causes clamping.
        table.increment_position(-10);
        assert_eq!(table.current_index(), 0);
        assert_eq!(table.state.scroll_direction, ScrollDirection::Up);
        assert_eq!(table.current_item(), Some(&TestType { index: 0 }));

        // Make sure that overscrolling down causes clamping.
        table.increment_position(100);
        assert_eq!(table.current_index(), 4);
        assert_eq!(table.state.scroll_direction, ScrollDirection::Down);
        assert_eq!(table.current_item(), Some(&TestType { index: 4 }));
    }

    /// A test to ensure that scroll offsets are correctly handled when we "lose" rows.
    #[test]
    fn test_lose_data() {
        let mut table = create_test_table();
        table.set_data((0..=4).map(|index| TestType { index }).collect::<Vec<_>>());

        table.set_position(4);
        assert_eq!(table.current_index(), 4);
        assert_eq!(table.state.scroll_direction, ScrollDirection::Down);
        assert_eq!(table.current_item(), Some(&TestType { index: 4 }));

        table.set_data((0..=2).map(|index| TestType { index }).collect::<Vec<_>>());
        assert_eq!(table.current_index(), 2);
        assert_eq!(table.state.scroll_direction, ScrollDirection::Down);
        assert_eq!(table.current_item(), Some(&TestType { index: 2 }));
    }
}
