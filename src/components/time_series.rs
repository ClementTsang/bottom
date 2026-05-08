mod auto_y;
mod percent;

use std::{
    borrow::Cow,
    cmp::{max, min},
    time::{Duration, Instant},
};

pub use auto_y::*;
pub use percent::*;
use tui::{style::Style, symbols::Marker, widgets::BorderType};

use crate::canvas::components::time_series::{
    AxisBound, ChartScaling, GraphData, LegendConstraints, LegendPosition, TimeGraph,
};

const STALE_MIN_MILLISECONDS: u64 = Duration::from_secs(30).as_millis() as u64;

/// Configuration values for a [`TimeseriesState`], sourced from [`crate::app::AppConfigFields`].
#[derive(Copy, Clone, Debug)]
pub struct TimeseriesConfig {
    pub time_interval: u64,
    pub retention_ms: u64,
    pub autohide_time: bool,
    pub default_time_value: u64,
}

/// A time_series graph widget displays data over a period of time.
pub struct TimeseriesState {
    config: TimeseriesConfig,
    current_display_time: u64,
    autohide_timer: Option<Instant>,
}

impl TimeseriesState {
    /// Create a new [`TimeseriesState`] using the given config.
    pub fn new(config: TimeseriesConfig) -> Self {
        Self {
            current_display_time: config.default_time_value,
            config,
            autohide_timer: None,
        }
    }

    /// Set the autohide timer.
    pub fn with_autohide_timer(mut self, autohide_timer: Option<Instant>) -> Self {
        self.autohide_timer = autohide_timer;
        self
    }

    /// Get a mutable reference to the autohide timer.
    pub fn autohide_timer_mut(&mut self) -> &mut Option<Instant> {
        &mut self.autohide_timer
    }

    /// Get the current display time.
    fn current_display_time(&self) -> u64 {
        self.current_display_time
    }

    /// Zoom in on the x-axis (reducing the time range shown).
    pub fn zoom_in(&mut self) {
        let new_time = self
            .current_display_time
            .saturating_sub(self.config.time_interval);
        self.current_display_time = max(new_time, STALE_MIN_MILLISECONDS);
        self.maybe_start_autohide();
    }

    /// Zoom out on the x-axis (increasing the time range shown).
    pub fn zoom_out(&mut self) {
        let new_time = self
            .current_display_time
            .saturating_add(self.config.time_interval);
        self.current_display_time = min(new_time, self.config.retention_ms);
        self.maybe_start_autohide();
    }

    /// Reset the zoom level to the default.
    pub fn reset_zoom(&mut self) {
        self.current_display_time = self.config.default_time_value;
        self.maybe_start_autohide();
    }

    fn maybe_start_autohide(&mut self) {
        if self.config.autohide_time {
            self.autohide_timer = Some(Instant::now());
        }
    }
}

/// Per-render context passed to component draw methods. Carries everything that
/// varies each frame but is not persistent state.
pub struct GraphDrawCtx<'a> {
    pub title: Cow<'a, str>,
    pub border_style: Style,
    pub title_style: Style,
    pub graph_style: Style,
    pub general_widget_style: Style,
    pub border_type: BorderType,
    pub marker: Marker,
    pub hide_x_labels: bool,
    pub is_selected: bool,
    pub is_expanded: bool,
    pub legend_position: Option<LegendPosition>,
    pub legend_constraints: Option<LegendConstraints>,
}

#[cfg(test)]
mod time_series_tests {
    use super::*;

    const BASE_CONFIG: TimeseriesConfig = TimeseriesConfig {
        time_interval: 15_000,
        retention_ms: 300_000,
        autohide_time: false,
        default_time_value: 60_000,
    };

    fn state_at(display_time: u64, cfg: TimeseriesConfig) -> TimeseriesState {
        TimeseriesState {
            config: cfg,
            current_display_time: display_time,
            autohide_timer: None,
        }
    }

    #[test]
    fn zoom_in_decreases_display_time() {
        let mut state = state_at(60_000, BASE_CONFIG);
        state.zoom_in();
        assert_eq!(state.current_display_time, 45_000);
    }

    #[test]
    fn zoom_in_clamps_at_minimum() {
        let mut state = state_at(35_000, BASE_CONFIG);
        state.zoom_in();
        assert_eq!(state.current_display_time, STALE_MIN_MILLISECONDS);
    }

    #[test]
    fn zoom_out_increases_display_time() {
        let mut state = state_at(60_000, BASE_CONFIG);
        state.zoom_out();
        assert_eq!(state.current_display_time, 75_000);
    }

    #[test]
    fn zoom_out_clamps_at_retention() {
        let mut state = state_at(290_000, BASE_CONFIG);
        state.zoom_out();
        assert_eq!(state.current_display_time, 300_000);
    }

    #[test]
    fn reset_zoom_restores_default() {
        let mut state = state_at(120_000, BASE_CONFIG);
        state.reset_zoom();
        assert_eq!(state.current_display_time, 60_000);
    }

    #[test]
    fn autohide_armed_on_change() {
        let mut state = state_at(
            60_000,
            TimeseriesConfig {
                autohide_time: true,
                ..BASE_CONFIG
            },
        );
        state.zoom_in();
        assert!(state.autohide_timer.is_some());
    }
}

#[cfg(test)]
mod graph_height_tests {
    use timeless::data::ChunkedData;

    use super::*;

    fn build(times: &[Instant], values: &[f64]) -> ChunkedData<f64> {
        assert_eq!(times.len(), values.len());
        let mut data = ChunkedData::default();
        for &v in values {
            data.push(v);
        }
        data
    }

    #[test]
    fn picks_max_across_sources() {
        let mut cache = GraphHeightCache::default();
        let now = Instant::now();
        let times = vec![
            now - Duration::from_millis(300),
            now - Duration::from_millis(200),
            now - Duration::from_millis(100),
            now,
        ];
        let a = build(&times, &[1.0, 2.0, 3.0, 4.0]);
        let b = build(&times, &[10.0, 5.0, 0.5, 0.25]);

        let result = cache.get_or_update(&now, 1_000, [&a, &b].into_iter(), &times);
        assert_eq!(result, 10.0);
    }

    #[test]
    fn cache_hit_skips_older_points() {
        // On a cache hit, only points after `right_edge` are rescanned. So if
        // we pass a fresh source whose only large values are *older* than the
        // previous `last_time`, the cached max should still win.
        let mut cache = GraphHeightCache::default();
        let now = Instant::now();
        let times = vec![
            now - Duration::from_millis(200),
            now - Duration::from_millis(100),
            now,
        ];
        let first = build(&times, &[3.0, 5.0, 7.0]);

        let first_result = cache.get_or_update(&now, 10_000, [&first].into_iter(), &times);
        assert_eq!(first_result, 7.0);

        // Older points carry huge values; newest is small. A full rescan would
        // return 1000.0 — a true cache hit returns the cached 7.0.
        let second = build(&times, &[1000.0, 999.0, 4.0]);
        let second_result = cache.get_or_update(&now, 10_000, [&second].into_iter(), &times);
        assert_eq!(second_result, 7.0);
    }

    #[test]
    fn cache_invalidates_on_period_change() {
        // First call uses a small window that excludes the older high value.
        // Second call uses a larger window that should include it; a stale
        // cache would miss it and return the previous max.
        let mut cache = GraphHeightCache::default();
        let now = Instant::now();
        let times = vec![
            now - Duration::from_millis(500),
            now - Duration::from_millis(50),
            now,
        ];
        let data = build(&times, &[100.0, 5.0, 7.0]);

        let first = cache.get_or_update(&now, 100, [&data].into_iter(), &times);
        assert_eq!(first, 7.0);

        let second = cache.get_or_update(&now, 1_000, [&data].into_iter(), &times);
        assert_eq!(second, 100.0);
    }
}
