pub mod battery_info;
pub mod cpu_graph;
pub mod disk_table;
pub mod mem_graph;
pub mod network_graph;
pub mod process_table;
pub mod temperature_graph;
pub mod temperature_table;

use std::time::{Duration, Instant};

pub use battery_info::*;
pub use cpu_graph::*;
pub use disk_table::*;
pub use mem_graph::*;
pub use network_graph::*;
pub use process_table::*;
pub use temperature_graph::*;
pub use temperature_table::*;
use timeless::data::ChunkedData;

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
mod tests {
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
