use serde::Deserialize;

use super::{IgnoreList, disk_io_graph::DiskGraphLegend};

/// Disk space graph configuration.
#[derive(Clone, Debug, Default, Deserialize)]
#[cfg_attr(feature = "generate_schema", derive(schemars::JsonSchema))]
#[cfg_attr(test, serde(deny_unknown_fields), derive(PartialEq, Eq))]
pub(crate) struct DiskSpaceGraphConfig {
    /// Whether to label legend entries by device name or mount point. Defaults to disk name.
    pub(crate) legend: Option<DiskGraphLegend>,

    /// Where to position the legend within the widget.
    pub(crate) legend_position: Option<String>,

    /// An optional list of device names to include or exclude.
    pub(crate) name_filter: Option<IgnoreList>,
}
