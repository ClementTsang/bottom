use std::num::NonZeroU16;

use tui::{layout::Rect, widgets::TableState};

#[derive(Debug, Copy, Clone, PartialEq, Eq, Default)]
pub enum ScrollDirection {
    // UP means scrolling up --- this usually DECREMENTS
    Up,

    // DOWN means scrolling down --- this usually INCREMENTS
    #[default]
    Down,
}

/// Internal state representation of a [`DataTable`](super::DataTable).
pub struct DataTableState {
    /// The index from where to start displaying the rows.
    pub display_start_index: usize,

    /// The current scroll position.
    pub current_index: usize,

    /// The direction of the last attempted scroll.
    pub scroll_direction: ScrollDirection,

    /// ratatui's internal table state.
    pub table_state: TableState,

    /// The calculated widths.
    pub calculated_widths: Vec<NonZeroU16>,

    /// The current inner [`Rect`].
    pub inner_rect: Rect,
}

impl Default for DataTableState {
    fn default() -> Self {
        Self {
            display_start_index: 0,
            current_index: 0,
            scroll_direction: ScrollDirection::Down,
            calculated_widths: vec![],
            table_state: TableState::default(),
            inner_rect: Rect::default(),
        }
    }
}

impl DataTableState {
    /// Gets the starting position of a table.
    pub fn get_start_position(&mut self, num_rows: usize, is_force_redraw: bool) {
        let start_index = if is_force_redraw {
            0
        } else {
            self.display_start_index
        };
        let current_scroll_position = self.current_index;
        let scroll_direction = self.scroll_direction;

        self.display_start_index = match scroll_direction {
            ScrollDirection::Down => {
                if current_scroll_position < start_index + num_rows {
                    // If, using the current scroll position, we can see the element
                    // (so within that and + num_rows) just reuse the current previously
                    // scrolled position.
                    start_index
                } else if current_scroll_position >= num_rows {
                    // If the current position past the last element visible in the list,
                    // then skip until we can see that element.
                    current_scroll_position - num_rows + 1
                } else {
                    // Else, if it is not past the last element visible, do not omit anything.
                    0
                }
            }
            ScrollDirection::Up => {
                if current_scroll_position <= start_index {
                    // If it's past the first element, then show from that element downwards
                    current_scroll_position
                } else if current_scroll_position >= start_index + num_rows {
                    current_scroll_position - num_rows + 1
                } else {
                    start_index
                }
            }
        };
    }
}
