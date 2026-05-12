use serde::Deserialize;

use super::IgnoreList;

/// Network configuration.
#[derive(Clone, Debug, Default, Deserialize)]
#[cfg_attr(feature = "generate_schema", derive(schemars::JsonSchema))]
#[cfg_attr(test, serde(deny_unknown_fields), derive(PartialEq, Eq))]
pub(crate) struct NetworkGraphConfig {
    /// A filter over the network interface names.
    pub(crate) interface_filter: Option<IgnoreList>,

    /// Displays packet rate and average packet size info.
    pub(crate) show_packets: Option<bool>,

    // TODO: We probably want to make this an enum...? If we want to also support external legends
    // (e.g. table-style, list-style) then we probably need a new system outright.
    /// Where to place the legend for the network chart widget.
    pub(crate) legend_position: Option<String>,

    /// Displays the network widget using bytes. Defaults to bits.
    pub(crate) use_bytes: Option<bool>,

    /// Displays the network widget with a log scale. Defaults to a non-log scale.
    pub(crate) use_log: Option<bool>,

    /// Displays the network widget with a binary prefix (e.g. kibibits) rather than a decimal
    /// prefix (e.g. kilobits). Defaults to decimal prefixes.
    pub(crate) use_binary_prefix: Option<bool>,
}
