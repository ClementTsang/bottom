//! Memory graph configuration.

use serde::Deserialize;

/// Memory-related configuration file options.
#[derive(Clone, Debug, Default, Deserialize)]
#[cfg_attr(feature = "generate_schema", derive(schemars::JsonSchema))]
#[cfg_attr(test, serde(deny_unknown_fields), derive(PartialEq, Eq))]
pub struct MemoryGraphConfig {
    // TODO: We probably want to make this an enum...? If we want to also support external legends
    // (e.g. table-style, list-style) then we probably need a new system outright.
    /// Where to place the legend for the memory chart widget.
    pub(crate) legend_position: Option<String>,

    /// Whether to collect and display cache and buffer memory. Not available on Windows.
    pub(crate) cache_memory: Option<bool>,

    /// Whether to subtract freeable ARC from memory usage.
    pub(crate) free_arc: Option<bool>,
}
