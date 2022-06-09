use tui::layout::Rect;

use super::{ScrollDirection, TableStyling};

pub enum SelectionState {
    NotSelected,
    Selected,
    Expanded,
}

/// A [`DrawInfo`] is information required on each draw call.
pub struct DrawInfo {
    pub styling: TableStyling,
    pub loc: Rect,
    pub force_redraw: bool,
    pub recalculate_column_widths: bool,
    pub selection_state: SelectionState,
}

impl DrawInfo {
    pub fn is_on_widget(&self) -> bool {
        matches!(self.selection_state, SelectionState::Selected)
            || matches!(self.selection_state, SelectionState::Expanded)
    }

    pub fn is_expanded(&self) -> bool {
        matches!(self.selection_state, SelectionState::Expanded)
    }
}

/// Gets the starting position of a table.
pub fn get_start_position(
    num_rows: usize, scroll_direction: &ScrollDirection, mut scroll_position_bar: usize,
    currently_selected_position: usize, is_force_redraw: bool,
) -> usize {
    if is_force_redraw {
        scroll_position_bar = 0;
    }

    match scroll_direction {
        ScrollDirection::Down => {
            if currently_selected_position < scroll_position_bar + num_rows {
                // If, using previous_scrolled_position, we can see the element
                // (so within that and + num_rows) just reuse the current previously scrolled position
                scroll_position_bar
            } else if currently_selected_position >= num_rows {
                // Else if the current position past the last element visible in the list, omit
                // until we can see that element
                currently_selected_position - num_rows + 1
            } else {
                // Else, if it is not past the last element visible, do not omit anything
                0
            }
        }
        ScrollDirection::Up => {
            if currently_selected_position <= scroll_position_bar {
                // If it's past the first element, then show from that element downwards
                currently_selected_position
            } else if currently_selected_position >= scroll_position_bar + num_rows {
                currently_selected_position - num_rows + 1
            } else {
                scroll_position_bar
            }
        }
    }
}
