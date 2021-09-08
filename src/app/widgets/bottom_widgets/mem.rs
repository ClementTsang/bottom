use std::{borrow::Cow, collections::HashMap, time::Instant};

use crossterm::event::{KeyEvent, MouseEvent};
use tui::{backend::Backend, layout::Rect};

use crate::{
    app::{event::WidgetEventResult, time_graph::TimeGraphData, DataCollection},
    app::{Component, TimeGraph, Widget},
    data_conversion::{convert_mem_data_points, convert_mem_labels, convert_swap_data_points},
    options::layout_options::LayoutRule,
};

pub struct MemWidgetState {
    pub current_display_time: u64,
    pub autohide_timer: Option<Instant>,
}

#[derive(Default)]
pub struct MemState {
    pub force_update: Option<u64>,
    pub widget_states: HashMap<u64, MemWidgetState>,
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
    width: LayoutRule,
    height: LayoutRule,
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
            width: LayoutRule::default(),
            height: LayoutRule::default(),
        }
    }

    /// Sets the width.
    pub fn width(mut self, width: LayoutRule) -> Self {
        self.width = width;
        self
    }

    /// Sets the height.
    pub fn height(mut self, height: LayoutRule) -> Self {
        self.height = height;
        self
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
        let block = self.block().selected(selected).build(painter);

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

    fn width(&self) -> LayoutRule {
        self.width
    }

    fn height(&self) -> LayoutRule {
        self.height
    }
}
