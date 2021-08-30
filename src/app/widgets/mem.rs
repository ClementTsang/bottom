use std::{borrow::Cow, collections::HashMap, time::Instant};

use crossterm::event::{KeyEvent, MouseEvent};
use tui::{
    backend::Backend,
    layout::Rect,
    widgets::{Block, Borders},
};

use crate::{
    app::{event::WidgetEventResult, time_graph::TimeGraphData, DataCollection},
    data_conversion::{convert_mem_data_points, convert_mem_labels, convert_swap_data_points},
};

use super::{Component, TimeGraph, Widget};

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

#[derive(Default)]
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
    mem_labels: Option<(String, String)>,
    swap_labels: Option<(String, String)>,
    mem_data: Vec<(f64, f64)>,
    swap_data: Vec<(f64, f64)>,
    bounds: Rect,
}

impl MemGraph {
    /// Creates a new [`MemGraph`].
    pub fn new(graph: TimeGraph) -> Self {
        Self {
            graph,
            mem_labels: Default::default(),
            swap_labels: Default::default(),
            mem_data: Default::default(),
            swap_data: Default::default(),
            bounds: Rect::default(),
        }
    }
}

impl Component for MemGraph {
    fn handle_key_event(&mut self, event: KeyEvent) -> WidgetEventResult {
        self.graph.handle_key_event(event)
    }

    fn handle_mouse_event(&mut self, event: MouseEvent) -> WidgetEventResult {
        self.graph.handle_mouse_event(event)
    }

    fn bounds(&self) -> Rect {
        self.bounds
    }

    fn set_bounds(&mut self, new_bounds: Rect) {
        self.bounds = new_bounds;
    }
}

impl Widget for MemGraph {
    fn get_pretty_name(&self) -> &'static str {
        "Memory"
    }

    fn draw<B: Backend>(
        &mut self, painter: &crate::canvas::Painter, f: &mut tui::Frame<'_, B>, area: Rect,
        selected: bool,
    ) {
        let block = Block::default()
            .border_style(if selected {
                painter.colours.highlighted_border_style
            } else {
                painter.colours.border_style
            })
            .borders(Borders::ALL);

        let mut chart_data = Vec::with_capacity(2);
        if let Some((label_percent, label_frac)) = &self.mem_labels {
            let mem_label = format!("RAM:{}{}", label_percent, label_frac);
            chart_data.push(TimeGraphData {
                data: &self.mem_data,
                label: Some(mem_label.into()),
                style: painter.colours.ram_style,
            });
        }
        if let Some((label_percent, label_frac)) = &self.swap_labels {
            let swap_label = format!("SWP:{}{}", label_percent, label_frac);
            chart_data.push(TimeGraphData {
                data: &self.swap_data,
                label: Some(swap_label.into()),
                style: painter.colours.swap_style,
            });
        }

        const Y_BOUNDS: [f64; 2] = [0.0, 100.5];
        let y_bound_labels: [Cow<'static, str>; 2] = ["0%".into(), "100%".into()];

        self.graph.draw_tui_chart(
            painter,
            f,
            &chart_data,
            &y_bound_labels,
            Y_BOUNDS,
            false,
            block,
            area,
        );
    }

    fn update_data(&mut self, data_collection: &DataCollection) {
        self.mem_data = convert_mem_data_points(data_collection, false); // TODO: I think the "is_frozen" part is useless... it's always false now.
        self.swap_data = convert_swap_data_points(data_collection, false);
        let (memory_labels, swap_labels) = convert_mem_labels(data_collection);

        self.mem_labels = memory_labels;
        self.swap_labels = swap_labels;
    }
}
