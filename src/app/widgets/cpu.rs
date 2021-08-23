use std::{collections::HashMap, time::Instant};

use crossterm::event::{KeyEvent, MouseEvent};
use tui::layout::Rect;

use crate::app::event::EventResult;

use super::{
    does_point_intersect_rect, AppScrollWidgetState, CanvasTableWidthState, Component, TextTable,
    TimeGraph, Widget,
};

pub struct CpuWidgetState {
    pub current_display_time: u64,
    pub is_legend_hidden: bool,
    pub autohide_timer: Option<Instant>,
    pub scroll_state: AppScrollWidgetState,
    pub is_multi_graph_mode: bool,
    pub table_width_state: CanvasTableWidthState,
}

impl CpuWidgetState {
    pub fn init(current_display_time: u64, autohide_timer: Option<Instant>) -> Self {
        CpuWidgetState {
            current_display_time,
            is_legend_hidden: false,
            autohide_timer,
            scroll_state: AppScrollWidgetState::default(),
            is_multi_graph_mode: false,
            table_width_state: CanvasTableWidthState::default(),
        }
    }
}

pub struct CpuState {
    pub force_update: Option<u64>,
    pub widget_states: HashMap<u64, CpuWidgetState>,
}

impl CpuState {
    pub fn init(widget_states: HashMap<u64, CpuWidgetState>) -> Self {
        CpuState {
            force_update: None,
            widget_states,
        }
    }

    pub fn get_mut_widget_state(&mut self, widget_id: u64) -> Option<&mut CpuWidgetState> {
        self.widget_states.get_mut(&widget_id)
    }

    pub fn get_widget_state(&self, widget_id: u64) -> Option<&CpuWidgetState> {
        self.widget_states.get(&widget_id)
    }
}

enum CpuGraphSelection {
    Graph,
    Legend,
    None,
}

/// Whether the [`CpuGraph`]'s legend is placed on the left or right.
pub enum CpuGraphLegendPosition {
    Left,
    Right,
}

/// A widget designed to show CPU usage via a graph, along with a side legend implemented as a [`TextTable`].
pub struct CpuGraph {
    graph: TimeGraph,
    legend: TextTable,
    pub legend_position: CpuGraphLegendPosition,

    bounds: Rect,
    selected: CpuGraphSelection,
}

impl CpuGraph {
    /// Creates a new [`CpuGraph`].
    pub fn new(
        graph: TimeGraph, legend: TextTable, legend_position: CpuGraphLegendPosition,
    ) -> Self {
        Self {
            graph,
            legend,
            legend_position,
            bounds: Rect::default(),
            selected: CpuGraphSelection::None,
        }
    }
}

impl Component for CpuGraph {
    fn handle_key_event(&mut self, event: KeyEvent) -> EventResult {
        match self.selected {
            CpuGraphSelection::Graph => self.graph.handle_key_event(event),
            CpuGraphSelection::Legend => self.legend.handle_key_event(event),
            CpuGraphSelection::None => EventResult::NoRedraw,
        }
    }

    fn handle_mouse_event(&mut self, event: MouseEvent) -> EventResult {
        let global_x = event.column;
        let global_y = event.row;

        if does_point_intersect_rect(global_x, global_y, self.graph.bounds()) {
            self.selected = CpuGraphSelection::Graph;
            self.graph.handle_mouse_event(event)
        } else if does_point_intersect_rect(global_x, global_y, self.legend.bounds()) {
            self.selected = CpuGraphSelection::Legend;
            self.legend.handle_mouse_event(event)
        } else {
            EventResult::NoRedraw
        }
    }

    fn bounds(&self) -> Rect {
        self.bounds
    }

    fn set_bounds(&mut self, new_bounds: Rect) {
        self.bounds = new_bounds;
    }
}

impl Widget for CpuGraph {
    fn get_pretty_name(&self) -> &'static str {
        "CPU"
    }
}
