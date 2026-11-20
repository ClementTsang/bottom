//! GPU usage graph configuration.

use serde::Deserialize;

/// GPU graph-related configuration file options.
#[derive(Clone, Debug, Default, Deserialize)]
#[cfg_attr(feature = "generate_schema", derive(schemars::JsonSchema))]
#[cfg_attr(test, serde(deny_unknown_fields), derive(PartialEq, Eq))]
pub struct GpuUsageGraphConfig {
    // TODO: We probably want to make this an enum...? If we want to also
    // support external legends (e.g. table-style, list-style) then we probably
    // need a new system outright.
    /// Where to place the legend for the GPU usage chart widget.
    pub(crate) legend_position: Option<String>,

    /// Whether to use short GPU names in the GPU usage widget.
    pub(crate) short_gpu_names: Option<bool>,
}
