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
