use serde::Deserialize;

use crate::options::DiskWidgetColumn;

use super::IgnoreList;

/// Disk configuration.
#[derive(Clone, Debug, Default, Deserialize)]
#[cfg_attr(feature = "generate_schema", derive(schemars::JsonSchema))]
#[cfg_attr(test, serde(deny_unknown_fields), derive(PartialEq, Eq))]
pub(crate) struct DiskConfig {
    /// A filter over the disk names.
    pub(crate) name_filter: Option<IgnoreList>,

    /// A filter over the mount names.
    pub(crate) mount_filter: Option<IgnoreList>,

    /// A list of disk widget columns.
    #[serde(default)]
    pub(crate) columns: Vec<DiskWidgetColumn>, // TODO: make this more composable(?) in the future, we might need to rethink how it's done for custom widgets
}
