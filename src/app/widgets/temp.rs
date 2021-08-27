use std::collections::HashMap;

use crossterm::event::{KeyEvent, MouseEvent};
use tui::{
    backend::Backend,
    layout::Rect,
    widgets::{Block, Borders},
    Frame,
};

use crate::{
    app::{event::EventResult, text_table::Column},
    canvas::{DisplayableData, Painter},
};

use super::{AppScrollWidgetState, CanvasTableWidthState, Component, TextTable, Widget};

pub struct TempWidgetState {
    pub scroll_state: AppScrollWidgetState,
    pub table_width_state: CanvasTableWidthState,
}

impl TempWidgetState {
    pub fn init() -> Self {
        TempWidgetState {
            scroll_state: AppScrollWidgetState::default(),
            table_width_state: CanvasTableWidthState::default(),
        }
    }
}

#[derive(Default)]
pub struct TempState {
    pub widget_states: HashMap<u64, TempWidgetState>,
}

impl TempState {
    pub fn init(widget_states: HashMap<u64, TempWidgetState>) -> Self {
        TempState { widget_states }
    }

    pub fn get_mut_widget_state(&mut self, widget_id: u64) -> Option<&mut TempWidgetState> {
        self.widget_states.get_mut(&widget_id)
    }

    pub fn get_widget_state(&self, widget_id: u64) -> Option<&TempWidgetState> {
        self.widget_states.get(&widget_id)
    }
}

/// A table displaying disk data.  Essentially a wrapper around a [`TextTable`].
pub struct TempTable {
    table: TextTable,
    bounds: Rect,
}

impl Default for TempTable {
    fn default() -> Self {
        let table = TextTable::new(vec![
            Column::new_flex("Sensor", None, false, 0.8),
            Column::new_hard("Temp", None, false, Some(4)),
        ])
        .left_to_right(false);

        Self {
            table,
            bounds: Rect::default(),
        }
    }
}

impl Component for TempTable {
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

impl Widget for TempTable {
    fn get_pretty_name(&self) -> &'static str {
        "Temperature"
    }

    fn draw<B: Backend>(
        &mut self, painter: &Painter, f: &mut Frame<'_, B>, area: Rect, data: &DisplayableData,
        selected: bool,
    ) {
        let block = Block::default()
            .border_style(if selected {
                painter.colours.highlighted_border_style
            } else {
                painter.colours.border_style
            })
            .borders(Borders::ALL); // TODO: Also do the scrolling indicator!

        self.set_bounds(area);
        let draw_area = block.inner(area);
        let (table, widths, mut tui_state) =
            self.table
                .create_draw_table(painter, &data.temp_sensor_data, draw_area);

        let table = table.highlight_style(if selected {
            painter.colours.currently_selected_text_style
        } else {
            painter.colours.text_style
        });

        f.render_stateful_widget(table.block(block).widths(&widths), area, &mut tui_state);
    }
}
