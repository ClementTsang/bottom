use serde::Deserialize;

use super::IgnoreList;

/// Network configuration.
#[derive(Clone, Debug, Default, Deserialize)]
pub struct NetworkConfig {
    /// A filter over the network interface names.
    pub interface_filter: Option<IgnoreList>,
}
