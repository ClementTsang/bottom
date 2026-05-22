use std::time::Instant;

use crate::{
    components::time_series::{AutoYAxisTimeGraph, TimeseriesConfig},
    options::config::disk_io_graph::DiskGraphLegend,
};

/// Runtime state for a disk I/O graph widget.
pub struct DiskIoGraphWidgetState {
    /// The underlying time-series graph with automatic y-axis scaling.
    pub graph: AutoYAxisTimeGraph,
    /// Whether the read rate line is currently shown.
    pub show_read: bool,
    /// Whether the write rate line is currently shown.
    pub show_write: bool,
    /// Whether legend entries use device names or mount points.
    pub legend: DiskGraphLegend,
    /// Whether the y-axis uses a logarithmic scale.
    pub use_log: bool,
}

impl DiskIoGraphWidgetState {
    pub fn new(
        config: TimeseriesConfig, autohide_timer: Option<Instant>, show_read: bool,
        show_write: bool, legend: DiskGraphLegend, use_log: bool,
    ) -> Self {
        Self {
            graph: AutoYAxisTimeGraph::new(config, autohide_timer),
            show_read,
            show_write,
            legend,
            use_log,
        }
    }
}
