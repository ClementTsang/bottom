use serde::Deserialize;

use super::IgnoreList;

/// Whether the disk graph legend labels use device names or mount points.
#[derive(Clone, Debug, Default, Deserialize, PartialEq, Eq)]
#[cfg_attr(feature = "generate_schema", derive(schemars::JsonSchema))]
#[serde(rename_all = "snake_case")]
pub enum DiskGraphLegend {
    /// Label entries by kernel device name (e.g. `sda`, `nvme0n1`).
    #[default]
    Disk,
    /// Label entries by mount point (e.g. `/`, `/home`).
    Mount,
}

impl DiskGraphLegend {
    /// The name to show for a device: its mount point if requested and
    /// available, the device name otherwise.
    pub fn display_name<'a>(&self, name: &'a str, mount_point: Option<&'a str>) -> &'a str {
        match self {
            DiskGraphLegend::Disk => name,
            DiskGraphLegend::Mount => mount_point.unwrap_or(name),
        }
    }
}

/// Disk I/O graph configuration.
#[derive(Clone, Debug, Default, Deserialize)]
#[cfg_attr(feature = "generate_schema", derive(schemars::JsonSchema))]
#[cfg_attr(test, serde(deny_unknown_fields), derive(PartialEq, Eq))]
pub(crate) struct DiskIoGraphConfig {
    /// Whether to show the read rate line. Defaults to true.
    pub(crate) show_read: Option<bool>,

    /// Whether to show the write rate line. Defaults to true.
    pub(crate) show_write: Option<bool>,

    /// Whether to label legend entries by device name or mount point. Defaults to disk name.
    pub(crate) legend: Option<DiskGraphLegend>,

    /// Whether to use a logarithmic scale on the y-axis. Defaults to false.
    pub(crate) use_log: Option<bool>,

    /// Where to position the legend within the widget.
    pub(crate) legend_position: Option<String>,

    /// An optional list of device names to include or exclude.
    pub(crate) name_filter: Option<IgnoreList>,
}
