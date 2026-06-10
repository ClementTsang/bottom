use std::time::Instant;

use crate::{
    components::time_series::{PercentTimeGraph, TimeseriesConfig},
    options::config::disk_io_graph::DiskGraphLegend,
};

/// Runtime state for a disk space graph widget.
pub struct DiskSpaceGraphWidgetState {
    /// The underlying time-series graph component.
    pub graph: PercentTimeGraph,
    /// Whether legend entries use device names or mount points.
    pub legend: DiskGraphLegend,
}

impl DiskSpaceGraphWidgetState {
    pub fn new(
        config: TimeseriesConfig, autohide_timer: Option<Instant>, legend: DiskGraphLegend,
    ) -> Self {
        Self {
            graph: PercentTimeGraph::new(config, autohide_timer),
            legend,
        }
    }
}
