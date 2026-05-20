use std::time::Instant;

use crate::components::time_series::{PercentTimeGraph, TimeseriesConfig};

pub struct MemWidgetState {
    pub graph: PercentTimeGraph,
}

impl MemWidgetState {
    pub fn init(config: TimeseriesConfig, autohide_timer: Option<Instant>) -> Self {
        MemWidgetState {
            graph: PercentTimeGraph::new(config, autohide_timer),
        }
    }
}
