use std::collections::HashMap;

use crossterm::event::{KeyEvent, MouseEvent};
use tui::layout::Rect;

use crate::app::event::EventResult;

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

impl TempTable {
    /// Creates a new [`TempTable`].
    pub fn new(table: TextTable) -> Self {
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
}
