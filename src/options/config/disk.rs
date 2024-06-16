use serde::Deserialize;

use super::IgnoreList;

/// Disk configuration.
#[derive(Clone, Debug, Default, Deserialize)]
pub struct DiskConfig {
    /// A filter over the disk names.
    pub name_filter: Option<IgnoreList>,

    /// A filter over the mount names.
    pub mount_filter: Option<IgnoreList>,
}
