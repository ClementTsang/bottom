use std::{cmp::min, convert::TryInto};

use tui::widgets::TableState;

#[derive(Debug)]
pub enum ScrollDirection {
    // UP means scrolling up --- this usually DECREMENTS
    Up,

    // DOWN means scrolling down --- this usually INCREMENTS
    Down,
}

impl Default for ScrollDirection {
    fn default() -> Self {
        ScrollDirection::Down
    }
}

/// Internal state representation of a [`DataTable`](super::DataTable).
pub struct DataTableState {
    /// The index from where to start displaying the rows.
    pub display_row_start_index: usize,

    /// The current scroll position.
    pub current_scroll_position: usize,

    /// The direction of the last attempted scroll.
    pub scroll_direction: ScrollDirection,

    /// tui-rs' internal table state.
    pub table_state: TableState,
}

impl Default for DataTableState {
    fn default() -> Self {
        Self {
            display_row_start_index: 0,
            current_scroll_position: 0,
            scroll_direction: ScrollDirection::Down,
            table_state: TableState::default(),
        }
    }
}

impl DataTableState {
    /// Sets the scroll position to the first value.
    pub fn set_scroll_first(&mut self) {
        self.current_scroll_position = 0;
        self.scroll_direction = ScrollDirection::Up;
    }

    /// Sets the scroll position to the last value.
    pub fn set_scroll_last(&mut self, num_entries: usize) {
        self.current_scroll_position = num_entries.saturating_sub(1);
        self.scroll_direction = ScrollDirection::Down;
    }

    /// Updates the scroll position to be valid for the number of entries.
    pub fn update_num_entries(&mut self, num_entries: usize) {
        self.current_scroll_position =
            min(self.current_scroll_position, num_entries.saturating_sub(1));
    }

    /// Updates the scroll position if possible by a positive/negative offset. If there is a
    /// valid change, this function will also return the new position wrapped in an [`Option`].
    pub fn update_scroll_position(&mut self, change: i64, num_entries: usize) -> Option<usize> {
        if change == 0 {
            return None;
        }

        let csp: Result<i64, _> = self.current_scroll_position.try_into();
        if let Ok(csp) = csp {
            let proposed: Result<usize, _> = (csp + change).try_into();
            if let Ok(proposed) = proposed {
                if proposed < num_entries {
                    self.current_scroll_position = proposed;
                    if change < 0 {
                        self.scroll_direction = ScrollDirection::Up;
                    } else {
                        self.scroll_direction = ScrollDirection::Down;
                    }

                    return Some(self.current_scroll_position);
                }
            }
        }

        None
    }
}
