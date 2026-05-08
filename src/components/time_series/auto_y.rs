use std::{
    borrow::Cow,
    time::{Duration, Instant},
};

use timeless::data::ChunkedData;
use tui::{Frame, layout::Rect};

use super::{
    AxisBound, ChartScaling, GraphData, GraphDrawCtx, TimeGraph, TimeseriesConfig, TimeseriesState,
};

struct GraphHeightCacheInner {
    best_point: (Instant, f64),
    right_edge: Instant,
    period: u64,
}

#[derive(Default)]
pub struct GraphHeightCache {
    inner: Option<GraphHeightCacheInner>,
}

impl GraphHeightCache {
    /// Get the cached height if it exists, or update it otherwise.
    pub(crate) fn get_or_update<
        'a,
        F: Into<f64> + Clone + Copy + 'a,
        S: Iterator<Item = &'a ChunkedData<F>>,
    >(
        &mut self, last_time: &Instant, current_display_time: u64, sources: S, times: &[Instant],
    ) -> f64 {
        let visible_duration = Duration::from_millis(current_display_time);

        let (mut biggest, mut biggest_time, oldest_to_check) = if let Some(GraphHeightCacheInner {
            best_point,
            right_edge,
            period,
        }) = self.inner.as_ref()
            && *period == current_display_time
            && last_time.duration_since(best_point.0) < visible_duration
        {
            (best_point.1, best_point.0, *right_edge)
        } else {
            let visible_duration = Duration::from_millis(current_display_time);

            let visible_left_bound = match last_time.checked_sub(visible_duration) {
                Some(v) => v,
                None => {
                    // On some systems (like Windows) it can be possible that the
                    // current display time causes subtraction to fail if, for example,
                    // the uptime of the system is too low and current_display_time is
                    // too high. See https://github.com/ClementTsang/bottom/issues/1825.
                    //
                    // As such, we instead take the oldest visible time. This is a bit
                    // inefficient, but since it should only happen rarely, it's fine.
                    times
                        .iter()
                        .take_while(|t| last_time.duration_since(**t) < visible_duration)
                        .last()
                        .cloned()
                        .unwrap_or(*last_time)
                }
            };

            (0.0, visible_left_bound, visible_left_bound)
        };

        for source in sources {
            for (&time, &v) in source
                .iter_along_base(times)
                .rev()
                .take_while(|&(&time, _)| time >= oldest_to_check)
            {
                let v = v.into();
                if v > biggest {
                    biggest = v;
                    biggest_time = time;
                }
            }
        }

        self.inner = Some(GraphHeightCacheInner {
            best_point: (biggest_time, biggest),
            right_edge: *last_time,
            period: current_display_time,
        });

        biggest
    }
}

/// A time series graph that automatically adjusts the y-axis based on the data provided.
pub struct AutoYAxisTimeGraph {
    state: TimeseriesState,
    height_cache: GraphHeightCache,
}

impl AutoYAxisTimeGraph {
    pub(crate) fn new(config: TimeseriesConfig, autohide_timer: Option<Instant>) -> Self {
        AutoYAxisTimeGraph {
            state: TimeseriesState::new(config).with_autohide_timer(autohide_timer),
            height_cache: GraphHeightCache::default(),
        }
    }

    pub(crate) fn state_mut(&mut self) -> &mut TimeseriesState {
        &mut self.state
    }

    pub(crate) fn y_max<'a, F, S>(&mut self, sources: S, times: &[Instant]) -> f64
    where
        F: Into<f64> + Clone + Copy + 'a,
        S: Iterator<Item = &'a ChunkedData<F>>,
    {
        if let Some(last_time) = times.last() {
            self.height_cache
                .get_or_update(last_time, self.state.current_display_time(), sources, times)
        } else {
            0.0
        }
    }

    pub(crate) fn draw<F: Copy + Default + Into<f64>>(
        &self, f: &mut Frame<'_>, draw_loc: Rect, ctx: GraphDrawCtx<'_>, y_bounds: AxisBound,
        y_labels: &[Cow<'_, str>], scaling: ChartScaling, data: Vec<GraphData<'_, F>>,
    ) {
        TimeGraph {
            x_min: -(self.state.current_display_time() as f64),
            hide_x_labels: ctx.hide_x_labels,
            y_bounds,
            y_labels,
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
            scaling,
        }
        .draw(f, draw_loc, data)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn empty_sources_returns_zero() {
        let mut cache = GraphHeightCache::default();
        let last_time = Instant::now();
        let times: Vec<Instant> = vec![];
        let sources: Vec<ChunkedData<f64>> = vec![];

        let result = cache.get_or_update(&last_time, 1_000, sources.iter(), &times);
        assert_eq!(result, 0.0);
        assert!(cache.inner.is_some());
    }
}
