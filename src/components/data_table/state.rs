use tui::{layout::Rect, widgets::TableState};

#[derive(Debug, Copy, Clone)]
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
    pub display_start_index: usize,

    /// The current scroll position.
    pub current_scroll_position: usize,

    /// The direction of the last attempted scroll.
    pub scroll_direction: ScrollDirection,

    /// The calculated widths.
    pub calculated_widths: Vec<u16>,

    /// tui-rs' internal table state.
    pub table_state: TableState,

    /// The current inner [`Rect`].
    pub inner_rect: Rect,
}

impl Default for DataTableState {
    fn default() -> Self {
        Self {
            display_start_index: 0,
            current_scroll_position: 0,
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
        let mut start_index = self.display_start_index;
        let current_scroll_position = self.current_scroll_position;
        let scroll_direction = self.scroll_direction;

        if is_force_redraw {
            start_index = 0;
        }

        self.display_start_index = match scroll_direction {
            ScrollDirection::Down => {
                if current_scroll_position < start_index + num_rows {
                    // If, using previous_scrolled_position, we can see the element
                    // (so within that and + num_rows) just reuse the current previously scrolled position
                    start_index
                } else if current_scroll_position >= num_rows {
                    // Else if the current position past the last element visible in the list, omit
                    // until we can see that element
                    current_scroll_position - num_rows + 1
                } else {
                    // Else, if it is not past the last element visible, do not omit anything
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
