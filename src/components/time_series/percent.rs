use std::{borrow::Cow, time::Instant};

use tui::{Frame, layout::Rect};

use super::{
    AxisBound, ChartScaling, GraphData, GraphDrawCtx, TimeGraph, TimeseriesConfig, TimeseriesState,
};

/// A time series graph that expects data to be in a percentage format,
/// from 0% to 100%.
pub struct PercentTimeGraph {
    state: TimeseriesState,
}

impl PercentTimeGraph {
    pub(crate) fn new(config: TimeseriesConfig, autohide_timer: Option<Instant>) -> Self {
        PercentTimeGraph {
            state: TimeseriesState::new(config, autohide_timer),
        }
    }

    pub(crate) fn state_mut(&mut self) -> &mut TimeseriesState {
        &mut self.state
    }

    pub(crate) fn draw<F: Copy + Default + Into<f64>>(
        &self, f: &mut Frame<'_>, draw_loc: Rect, ctx: GraphDrawCtx<'_>,
        data: Vec<GraphData<'_, F>>,
    ) {
        const Y_BOUNDS: AxisBound = AxisBound::Max(100.5);
        const Y_LABELS: [Cow<'static, str>; 2] = [Cow::Borrowed("  0%"), Cow::Borrowed("100%")];

        TimeGraph {
            x_min: -(self.state.current_display_time() as f64),
            hide_x_labels: ctx.hide_x_labels,
            y_bounds: Y_BOUNDS,
            y_labels: &Y_LABELS,
            graph_style: ctx.graph_style,
            general_widget_style: ctx.general_widget_style,
            border_style: ctx.border_style,
            border_type: ctx.border_type,
            title: ctx.title,
            is_selected: ctx.is_selected,
            is_expanded: ctx.is_expanded,
            title_style: ctx.title_style,
            legend_position: ctx.legend_position,
            legend_constraints: ctx.legend_constraints,
            marker: ctx.marker,
            scaling: ChartScaling::Linear,
        }
        .draw(f, draw_loc, data)
    }
}
