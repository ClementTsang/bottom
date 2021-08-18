use std::{collections::HashMap, time::Instant};

use tui::layout::Rect;

use super::{TimeGraph, Widget};

pub struct NetWidgetState {
    pub current_display_time: u64,
    pub autohide_timer: Option<Instant>,
    // pub draw_max_range_cache: f64,
    // pub draw_labels_cache: Vec<String>,
    // pub draw_time_start_cache: f64,
    // TODO: Re-enable these when we move net details state-side!
    // pub unit_type: DataUnitTypes,
    // pub scale_type: AxisScaling,
}

impl NetWidgetState {
    pub fn init(
        current_display_time: u64,
        autohide_timer: Option<Instant>,
        // unit_type: DataUnitTypes,
        // scale_type: AxisScaling,
    ) -> Self {
        NetWidgetState {
            current_display_time,
            autohide_timer,
            // draw_max_range_cache: 0.0,
            // draw_labels_cache: vec![],
            // draw_time_start_cache: 0.0,
            // unit_type,
            // scale_type,
        }
    }
}
pub struct NetState {
    pub force_update: Option<u64>,
    pub widget_states: HashMap<u64, NetWidgetState>,
}

impl NetState {
    pub fn init(widget_states: HashMap<u64, NetWidgetState>) -> Self {
        NetState {
            force_update: None,
            widget_states,
        }
    }

    pub fn get_mut_widget_state(&mut self, widget_id: u64) -> Option<&mut NetWidgetState> {
        self.widget_states.get_mut(&widget_id)
    }

    pub fn get_widget_state(&self, widget_id: u64) -> Option<&NetWidgetState> {
        self.widget_states.get(&widget_id)
    }
}

struct NetGraphCache {
    max_range: f64,
    labels: Vec<String>,
    time_start: f64,
}

enum NetGraphCacheState {
    Uncached,
    Cached(NetGraphCache),
}

/// A widget denoting network usage via a graph. This version is self-contained within a single [`TimeGraph`];
/// if you need the older one that splits into two sections, use [`OldNetGraph`], which is built on a [`NetGraph`].
///
/// As of now, this is essentially just a wrapper around a [`TimeGraph`].
pub struct NetGraph {
    graph: TimeGraph,

    // Cached details; probably want to move at some point...
    draw_cache: NetGraphCacheState,
}

impl NetGraph {
    /// Creates a new [`NetGraph`].
    pub fn new(graph: TimeGraph) -> Self {
        Self {
            graph,
            draw_cache: NetGraphCacheState::Uncached,
        }
    }

    pub fn set_cache(&mut self, max_range: f64, labels: Vec<String>, time_start: f64) {
        self.draw_cache = NetGraphCacheState::Cached(NetGraphCache {
            max_range,
            labels,
            time_start,
        })
    }

    pub fn is_cached(&self) -> bool {
        match self.draw_cache {
            NetGraphCacheState::Uncached => false,
            NetGraphCacheState::Cached(_) => true,
        }
    }
}

impl Widget for NetGraph {
    type UpdateData = ();

    fn bounds(&self) -> Rect {
        self.graph.bounds()
    }

    fn set_bounds(&mut self, new_bounds: Rect) {
        self.graph.set_bounds(new_bounds);
    }

    fn handle_key_event(
        &mut self, event: crossterm::event::KeyEvent,
    ) -> crate::app::event::EventResult {
        self.graph.handle_key_event(event)
    }

    fn handle_mouse_event(
        &mut self, event: crossterm::event::MouseEvent,
    ) -> crate::app::event::EventResult {
        self.graph.handle_mouse_event(event)
    }
}

/// A widget denoting network usage via a graph and a separate, single row table. This is built on [`NetGraph`],
/// and the main difference is that it also contains a bounding box for the graph + text.
pub struct OldNetGraph {
    graph: NetGraph,
    bounds: Rect,
}

impl OldNetGraph {
    /// Creates a new [`OldNetGraph`].
    pub fn new(graph: NetGraph) -> Self {
        Self {
            graph,
            bounds: Rect::default(),
        }
    }
}

impl Widget for OldNetGraph {
    type UpdateData = ();

    fn bounds(&self) -> Rect {
        self.bounds
    }

    fn set_bounds(&mut self, new_bounds: Rect) {
        self.bounds = new_bounds;
    }

    fn handle_key_event(
        &mut self, event: crossterm::event::KeyEvent,
    ) -> crate::app::event::EventResult {
        self.graph.handle_key_event(event)
    }

    fn handle_mouse_event(
        &mut self, event: crossterm::event::MouseEvent,
    ) -> crate::app::event::EventResult {
        self.graph.handle_mouse_event(event)
    }
}
