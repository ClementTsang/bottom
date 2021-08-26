use std::collections::HashMap;

use crossterm::event::{KeyEvent, MouseEvent};
use tui::{backend::Backend, layout::Rect, widgets::Block, Frame};

use crate::{
    app::{event::EventResult, text_table::Column},
    canvas::{DisplayableData, Painter},
};

use super::{AppScrollWidgetState, CanvasTableWidthState, Component, TextTable, Widget};

pub struct DiskWidgetState {
    pub scroll_state: AppScrollWidgetState,
    pub table_width_state: CanvasTableWidthState,
}

impl DiskWidgetState {
    pub fn init() -> Self {
        DiskWidgetState {
            scroll_state: AppScrollWidgetState::default(),
            table_width_state: CanvasTableWidthState::default(),
        }
    }
}

#[derive(Default)]
pub struct DiskState {
    pub widget_states: HashMap<u64, DiskWidgetState>,
}

impl DiskState {
    pub fn init(widget_states: HashMap<u64, DiskWidgetState>) -> Self {
        DiskState { widget_states }
    }

    pub fn get_mut_widget_state(&mut self, widget_id: u64) -> Option<&mut DiskWidgetState> {
        self.widget_states.get_mut(&widget_id)
    }

    pub fn get_widget_state(&self, widget_id: u64) -> Option<&DiskWidgetState> {
        self.widget_states.get(&widget_id)
    }
}

/// A table displaying disk data.  Essentially a wrapper around a [`TextTable`].
pub struct DiskTable {
    table: TextTable,
    bounds: Rect,
}

impl Default for DiskTable {
    fn default() -> Self {
        let table = TextTable::new(vec![
            Column::new_flex("Disk", None, false, 0.2),
            Column::new_flex("Mount", None, false, 0.2),
            Column::new_hard("Used", None, false, Some(4)),
            Column::new_hard("Free", None, false, Some(6)),
            Column::new_hard("Total", None, false, Some(6)),
            Column::new_hard("R/s", None, false, Some(7)),
            Column::new_hard("W/s", None, false, Some(7)),
        ]);

        Self {
            table,
            bounds: Rect::default(),
        }
    }
}

impl Component for DiskTable {
    fn handle_key_event(&mut self, event: KeyEvent) -> EventResult {
        self.table.handle_key_event(event)
    }

    fn handle_mouse_event(&mut self, event: MouseEvent) -> EventResult {
        self.table.handle_mouse_event(event)
    }

    fn bounds(&self) -> Rect {
        self.bounds
    }

    fn set_bounds(&mut self, new_bounds: Rect) {
        self.bounds = new_bounds;
    }
}

impl Widget for DiskTable {
    fn get_pretty_name(&self) -> &'static str {
        "Disk"
    }

    fn draw<B: Backend>(
        &mut self, painter: &Painter, f: &mut Frame<'_, B>, area: Rect, block: Block<'_>,
        data: &DisplayableData,
    ) {
        let draw_area = block.inner(area);
        let (table, widths, mut tui_state) =
            self.table
                .create_draw_table(painter, &data.disk_data, draw_area);

        f.render_stateful_widget(table.block(block).widths(&widths), area, &mut tui_state);
    }
}
