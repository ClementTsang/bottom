use serde::Deserialize;

use super::IgnoreList;

/// Temperature configuration.
#[derive(Clone, Debug, Default, Deserialize)]
#[cfg_attr(feature = "generate_schema", derive(schemars::JsonSchema))]
#[cfg_attr(test, serde(deny_unknown_fields), derive(PartialEq))]
pub(crate) struct TempGraphConfig {
    /// A filter over the sensor names.
    pub(crate) sensor_filter: Option<IgnoreList>,

    /// The location of the graph's legend.
    #[serde(default)]
    pub(crate) legend_position: Option<String>,

    /// A max temperature value to clamp results to. If not set, there is no limit.
    /// Is in the unit of `temperature_type`.
    #[serde(default)]
    pub(crate) upper_limit: Option<f64>,
}
