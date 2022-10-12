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
    fn update_num_entries(&mut self) {
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
    pub fn set_position(&mut self, new_index: usize) {
        self.state.current_index = new_index.clamp(0, self.data.len().saturating_sub(1));
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
    // FIXME: Do all testing!
}
