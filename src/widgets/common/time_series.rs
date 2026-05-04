use std::{
    cmp::{max, min},
    time::{Duration, Instant},
};

use timeless::data::ChunkedData;

const STALE_MIN_MILLISECONDS: u64 = Duration::from_secs(30).as_millis() as u64;

/// A time_series graph widget displays data over a period of time.
pub struct TimeseriesState {
    current_display_time: u64,
    autohide_timer: Option<Instant>,
}

impl TimeseriesState {
    /// Create a new [`TimeseriesState`] that displays starting from `starting_time`.
    pub fn new(starting_time: u64) -> Self {
        Self {
            current_display_time: starting_time,
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
    pub fn current_display_time(&self) -> u64 {
        self.current_display_time
    }

    /// Zoom in on the x-axis (reducing the time range shown).
    pub fn zoom_in(&mut self, time_interval: u64, autohide_time: bool) {
        let new_time = self.current_display_time.saturating_sub(time_interval);

        self.current_display_time = max(new_time, STALE_MIN_MILLISECONDS);
        self.maybe_start_autohide(autohide_time);
    }

    /// Zoom out on the x-axis (increasing the time range shown).
    pub fn zoom_out(&mut self, time_interval: u64, retention_ms: u64, autohide_time: bool) {
        let new_time = self.current_display_time.saturating_add(time_interval);

        self.current_display_time = min(new_time, retention_ms);
        self.maybe_start_autohide(autohide_time);
    }

    /// Reset the zoom level to the default.
    pub fn reset_zoom(&mut self, default_time_value: u64, autohide_time: bool) {
        self.current_display_time = default_time_value;
        self.maybe_start_autohide(autohide_time);
    }

    /// Set the autohide timer if needed.
    fn maybe_start_autohide(&mut self, autohide_time: bool) {
        if autohide_time {
            self.autohide_timer = Some(Instant::now());
        }
    }
}

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
    /// Get the cached height if it exists, or set it otherwise.
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
                    // current display time
                    // causes subtraction to fail if, for example, the uptime of the
                    // system is too low and current_display_time is too high. See https://github.com/ClementTsang/bottom/issues/1825.
                    //
                    // As such, we instead take the oldest visible time. This is a
                    // bit inefficient, but
                    // since it should only happen rarely, it should be fine.
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

#[cfg(test)]
mod time_series_tests {
    use super::*;

    #[test]
    fn zoom_in_decreases_display_time() {
        let mut state = TimeseriesState {
            current_display_time: 60_000,
            autohide_timer: None,
        };

        state.zoom_in(15_000, false);
        assert_eq!(state.current_display_time, 45_000);
    }

    #[test]
    fn zoom_in_clamps_at_minimum() {
        let mut state = TimeseriesState {
            current_display_time: 35_000,
            autohide_timer: None,
        };

        state.zoom_in(15_000, false);
        assert_eq!(state.current_display_time, STALE_MIN_MILLISECONDS); // 30_000
    }

    #[test]
    fn zoom_out_increases_display_time() {
        let mut state = TimeseriesState {
            current_display_time: 60_000,
            autohide_timer: None,
        };

        state.zoom_out(15_000, 300_000, false);
        assert_eq!(state.current_display_time, 75_000);
    }

    #[test]
    fn zoom_out_clamps_at_retention() {
        let mut state = TimeseriesState {
            current_display_time: 290_000,
            autohide_timer: None,
        };

        state.zoom_out(15_000, 300_000, false);
        assert_eq!(state.current_display_time, 300_000);
    }

    #[test]
    fn reset_zoom_restores_default() {
        let mut state = TimeseriesState {
            current_display_time: 120_000,
            autohide_timer: None,
        };

        state.reset_zoom(60_000, false);
        assert_eq!(state.current_display_time, 60_000);
    }

    #[test]
    fn autohide_armed_on_change() {
        let mut state = TimeseriesState {
            current_display_time: 60_000,
            autohide_timer: None,
        };

        state.zoom_in(15_000, true);
        assert!(state.autohide_timer.is_some());
    }
}

#[cfg(test)]
mod graph_height_tests {
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
    fn empty_sources_returns_zero() {
        let mut cache = GraphHeightCache::default();
        let last_time = Instant::now();
        let times: Vec<Instant> = vec![];
        let sources: Vec<ChunkedData<f64>> = vec![];

        let result = cache.get_or_update(&last_time, 1_000, sources.iter(), &times);
        assert_eq!(result, 0.0);
        assert!(cache.inner.is_some());
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
