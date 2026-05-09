use std::time::Instant;

use crate::components::time_series::{AutoYAxisTimeGraph, TimeseriesConfig};

pub struct NetWidgetState {
    pub graph: AutoYAxisTimeGraph,
}

impl NetWidgetState {
    pub fn init(config: TimeseriesConfig, autohide_timer: Option<Instant>) -> Self {
        NetWidgetState {
            graph: AutoYAxisTimeGraph::new(config, autohide_timer),
        }
    }
}
