use crossterm::event::{KeyEvent, MouseEvent};
use tui::{backend::Backend, layout::Rect, Frame};

use crate::{
    app::{
        event::ComponentEventResult, text_table::SimpleColumn, widgets::tui_stuff::BlockBuilder,
        Component, TextTable,
    },
    canvas::Painter,
};

use super::sort_text_table::SortableColumn;

/// A sortable, scrollable table with columns.
pub struct SortMenu {
    /// The underlying table.
    table: TextTable,

    /// The bounds.
    bounds: Rect,
}

impl SortMenu {
    /// Creates a new [`SortMenu`].
    pub fn new(num_columns: usize) -> Self {
        let sort_menu_columns = vec![SimpleColumn::new_hard("Sort By".into(), None)];
        let mut sort_menu = TextTable::new(sort_menu_columns);
        sort_menu.set_num_items(num_columns);

        Self {
            table: sort_menu,
            bounds: Default::default(),
        }
    }

    pub fn try_show_gap(mut self, show_gap: bool) -> Self {
        self.table = self.table.try_show_gap(show_gap);
        self
    }

    /// Updates the index of the [`SortMenu`].
    pub fn set_index(&mut self, index: usize) {
        self.table.scrollable.set_index(index);
    }

    /// Returns the current index of the [`SortMenu`].
    pub fn current_index(&mut self) -> usize {
        self.table.scrollable.current_index()
    }

    /// Draws a [`tui::widgets::Table`] on screen corresponding to the sort columns of this [`SortableTextTable`].
    pub fn draw_sort_menu<B: Backend, C: SortableColumn>(
        &mut self, painter: &Painter, f: &mut Frame<'_, B>, columns: &[C], block: BlockBuilder,
        block_area: Rect,
    ) {
        self.set_bounds(block_area);

        let data = columns
            .iter()
            .map(|c| vec![(c.original_name().clone(), None, None)])
            .collect::<Vec<_>>();

        self.table
            .draw_tui_table(painter, f, &data, block, block_area, true, false);
    }
}

impl Component for SortMenu {
    fn bounds(&self) -> Rect {
        self.bounds
    }

    fn set_bounds(&mut self, new_bounds: Rect) {
        self.bounds = new_bounds;
    }

    fn handle_key_event(&mut self, event: KeyEvent) -> ComponentEventResult {
        self.table.handle_key_event(event)
    }

    fn handle_mouse_event(&mut self, event: MouseEvent) -> ComponentEventResult {
        self.table.handle_mouse_event(event)
    }
}
