use std::time::Instant;

use crate::components::time_series::{PercentTimeGraph, TimeseriesConfig};

/// Widget state for the GPU utilization graph.
pub struct GpuGraphWidgetState {
    pub graph: PercentTimeGraph,
}

impl GpuGraphWidgetState {
    pub fn init(config: TimeseriesConfig, autohide_timer: Option<Instant>) -> Self {
        GpuGraphWidgetState {
            graph: PercentTimeGraph::new(config, autohide_timer),
        }
    }
}
