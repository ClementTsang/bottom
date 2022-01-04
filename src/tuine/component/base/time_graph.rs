use std::borrow::Cow;

use tui::{style::Style, Frame};

use crate::tuine::{Bounds, DrawContext, Event, LayoutNode, StateContext, Status, TmpComponent};

pub struct TimeGraphData {
    pub data: Vec<(f64, f64)>,
    pub label: Option<Cow<'static, str>>,
    pub style: Style,
}

/// A [`TimeGraph`] is a component that indicates data in a graph form with the time being
/// the x-axis. It displays the most recent data at the right, with the recent data
/// being at the left.
pub struct TimeGraph {
    display_time: u64,
    default_time: u64,
    min_duration: u64,
    max_duration: u64,
    time_interval: u64,
    use_dot: bool,
    data: Vec<TimeGraphData>,
    y_bounds: [f64; 2],
    y_bound_labels: Vec<Cow<'static, str>>,
    reverse_order: bool,
}

impl TimeGraph {}

impl<Message> TmpComponent<Message> for TimeGraph {
    fn draw<Backend>(
        &mut self, state_ctx: &mut StateContext<'_>, draw_ctx: &DrawContext<'_>,
        frame: &mut Frame<'_, Backend>,
    ) where
        Backend: tui::backend::Backend,
    {
        todo!()
    }

    fn on_event(
        &mut self, state_ctx: &mut StateContext<'_>, draw_ctx: &DrawContext<'_>, event: Event,
        messages: &mut Vec<Message>,
    ) -> Status {
        Status::Ignored
    }

    fn layout(&self, bounds: Bounds, node: &mut LayoutNode) -> crate::tuine::Size {
        crate::tuine::Size {
            width: bounds.max_width,
            height: bounds.max_height,
        }
    }
}
