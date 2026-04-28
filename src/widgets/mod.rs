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

pub struct GraphHeightCache {
    pub best_point: (Instant, f64),
    pub right_edge: Instant,
    pub period: u64,
}

impl GraphHeightCache {
    pub(crate) fn get(
        &self, last_time: &Instant, current_display_time: u64,
    ) -> Option<(f64, Instant, Instant)> {
        let GraphHeightCache {
            best_point,
            right_edge,
            period,
        } = &self;

        let visible_duration = Duration::from_millis(current_display_time);

        if *period == current_display_time
            && last_time.duration_since(best_point.0) < visible_duration
        {
            Some((best_point.1, best_point.0, *right_edge))
        } else {
            None
        }
    }
}
