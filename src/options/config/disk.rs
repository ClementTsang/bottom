use serde::Deserialize;

use super::IgnoreList;

/// Disk configuration.
#[derive(Clone, Debug, Default, Deserialize)]
#[cfg_attr(feature = "generate_schema", derive(schemars::JsonSchema))]
#[cfg_attr(test, serde(deny_unknown_fields), derive(PartialEq, Eq))]
pub struct DiskConfig {
    /// A filter over the disk names.
    pub name_filter: Option<IgnoreList>,

    /// A filter over the mount names.
    pub mount_filter: Option<IgnoreList>,
}
