use std::{collections::HashMap, time::Instant};

use crossterm::event::{KeyEvent, MouseEvent};
use tui::layout::Rect;

use crate::app::event::EventResult;

use super::{TimeGraph, Widget};

pub struct MemWidgetState {
    pub current_display_time: u64,
    pub autohide_timer: Option<Instant>,
}

impl MemWidgetState {
    pub fn init(current_display_time: u64, autohide_timer: Option<Instant>) -> Self {
        MemWidgetState {
            current_display_time,
            autohide_timer,
        }
    }
}

pub struct MemState {
    pub force_update: Option<u64>,
    pub widget_states: HashMap<u64, MemWidgetState>,
}

impl MemState {
    pub fn init(widget_states: HashMap<u64, MemWidgetState>) -> Self {
        MemState {
            force_update: None,
            widget_states,
        }
    }

    pub fn get_mut_widget_state(&mut self, widget_id: u64) -> Option<&mut MemWidgetState> {
        self.widget_states.get_mut(&widget_id)
    }

    pub fn get_widget_state(&self, widget_id: u64) -> Option<&MemWidgetState> {
        self.widget_states.get(&widget_id)
    }
}

/// A widget that deals with displaying memory usage on a [`TimeGraph`].  Basically just a wrapper
/// around [`TimeGraph`] as of now.
pub struct MemGraph {
    graph: TimeGraph,
}

impl MemGraph {
    /// Creates a new [`MemGraph`].
    pub fn new(graph: TimeGraph) -> Self {
        Self { graph }
    }
}

impl Widget for MemGraph {
    type UpdateData = ();

    fn handle_key_event(&mut self, event: KeyEvent) -> EventResult {
        self.graph.handle_key_event(event)
    }

    fn handle_mouse_event(&mut self, event: MouseEvent) -> EventResult {
        self.graph.handle_mouse_event(event)
    }

    fn bounds(&self) -> Rect {
        self.graph.bounds()
    }

    fn set_bounds(&mut self, new_bounds: Rect) {
        self.graph.set_bounds(new_bounds);
    }
}
