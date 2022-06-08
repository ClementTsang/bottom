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

/// Internal state representation of a [`DataTable`].
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
