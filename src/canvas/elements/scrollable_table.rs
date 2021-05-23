#![allow(dead_code)]
#![allow(unused_variables)]

use tui::widgets::TableState;

use super::element::{Element, ElementBounds};

/// The state for a [`ScrollableTable`].
struct ScrollableTableState {
    tui_state: TableState,
}

/// A [`ScrollableTable`] is a stateful table [`Element`] with scrolling support.
pub struct ScrollableTable {
    bounds: ElementBounds,
    selected: bool,
    state: ScrollableTableState,
}

impl ScrollableTable {}

impl Element for ScrollableTable {
    fn draw<B: tui::backend::Backend>(
        &mut self, f: &mut tui::Frame<'_, B>, app_state: &crate::app::AppState,
        draw_loc: tui::layout::Rect, style: &crate::canvas::canvas_colours::CanvasColours,
    ) -> anyhow::Result<()> {
        Ok(())
    }

    fn recalculate_click_bounds(&mut self) {
        todo!()
    }

    fn click_bounds(&self) -> super::element::ElementBounds {
        self.bounds
    }

    fn is_selected(&self) -> bool {
        self.selected
    }

    fn select(&mut self) {
        self.selected = true;
    }

    fn unselect(&mut self) {
        self.selected = false;
    }
}
