use tui::layout::Rect;

pub enum SelectionState {
    NotSelected,
    Selected,
    Expanded,
}

/// A [`DrawInfo`] is information required on each draw call.
pub struct DrawInfo {
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
