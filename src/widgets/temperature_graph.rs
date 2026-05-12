//! Code around a temperature graph widget.

use std::time::Instant;

use crate::components::time_series::{AutoYAxisTimeGraph, TimeseriesConfig};

/// A time series graph widget displaying temperature usage over time.
pub struct TempGraphWidgetState {
    pub graph: AutoYAxisTimeGraph,
    pub max_temp: Option<f32>,
}

impl TempGraphWidgetState {
    pub fn new(
        config: TimeseriesConfig, autohide_timer: Option<Instant>, max_temp: Option<f32>,
    ) -> Self {
        TempGraphWidgetState {
            graph: AutoYAxisTimeGraph::new(config, autohide_timer),
            max_temp,
        }
    }
}
