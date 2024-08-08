use serde::Deserialize;

use super::IgnoreList;

/// Network configuration.
#[derive(Clone, Debug, Default, Deserialize)]
#[cfg_attr(feature = "generate_schema", derive(schemars::JsonSchema))]
#[cfg_attr(test, serde(deny_unknown_fields), derive(PartialEq, Eq))]
pub struct NetworkConfig {
    /// A filter over the network interface names.
    pub interface_filter: Option<IgnoreList>,
}
