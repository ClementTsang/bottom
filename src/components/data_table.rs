use std::{convert::TryInto, marker::PhantomData};

pub mod columns;
pub use columns::*;

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

pub mod sort;
pub use sort::*;

/// A [`DataTable`] is a component that displays data in a tabular form.
///
/// Note that [`DataTable`] supports a generic type `S`, bounded by [`SortType`]. This controls whether this table
/// expects sorted data or not, with two expected types:
///
/// - [`Unsortable`]: The default if otherwise not specified. This table does not expect sorted data.
/// - [`Sortable`]: This table expects sorted data, and there are helper functions to
///   facilitate things like sorting based on a selected column, shortcut column selection support, mouse column
///   selection support, etc.
pub struct DataTable<DataType: ToDataRow, T: ColumnDisplay = &'static str, S: SortType = Unsortable>
{
    pub columns: Vec<Column<T>>,
    pub state: DataTableState,
    pub props: DataTableProps,
    pub styling: DataTableStyling,
    sort_type: S,
    _pd: PhantomData<(DataType, S)>,
}

impl<DataType: ToDataRow, T: ColumnDisplay, S: SortType> DataTable<DataType, T, S> {
    /// Sets the scroll position to the first value.
    pub fn set_first(&mut self) {
        self.state.current_scroll_position = 0;
        self.state.scroll_direction = ScrollDirection::Up;
    }

    /// Sets the scroll position to the last value.
    pub fn set_last(&mut self, num_entries: usize) {
        self.state.current_scroll_position = num_entries.saturating_sub(1);
        self.state.scroll_direction = ScrollDirection::Down;
    }

    /// Updates the scroll position to be valid for the number of entries.
    pub fn update_num_entries(&mut self, num_entries: usize) {
        let max_pos = num_entries.saturating_sub(1);
        if self.state.current_scroll_position > max_pos {
            self.state.current_scroll_position = max_pos;
            self.reset_scroll_index();
        }
    }

    /// Increments the scroll position if possible by a positive/negative offset. If there is a
    /// valid change, this function will also return the new position wrapped in an [`Option`].
    pub fn increment_position(&mut self, change: i64, num_entries: usize) -> Option<usize> {
        if change == 0 {
            return None;
        }

        let csp: Result<i64, _> = self.state.current_scroll_position.try_into();
        if let Ok(csp) = csp {
            let proposed: Result<usize, _> = (csp + change).try_into();
            if let Ok(proposed) = proposed {
                if proposed < num_entries {
                    self.state.current_scroll_position = proposed;
                    self.state.scroll_direction = if change < 0 {
                        ScrollDirection::Up
                    } else {
                        ScrollDirection::Down
                    };

                    return Some(self.state.current_scroll_position);
                }
            }
        }

        None
    }

    /// Updates the scroll position to a selected index.
    pub fn set_position(&mut self, new_index: usize, num_entries: usize) {
        self.state.current_scroll_position = new_index.clamp(0, num_entries);
    }

    /// Resets the displayed start index to 0.
    fn reset_scroll_index(&mut self) {
        self.state.display_start_index = 0;
        self.state.scroll_direction = ScrollDirection::Down;
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
