use serde::Deserialize;

use super::IgnoreList;

/// Temperature graph configuration.
#[derive(Clone, Debug, Default, Deserialize)]
#[cfg_attr(feature = "generate_schema", derive(schemars::JsonSchema))]
#[cfg_attr(test, serde(deny_unknown_fields), derive(PartialEq))]
pub(crate) struct TempGraphConfig {
    /// A filter over the sensor names.
    pub(crate) sensor_filter: Option<IgnoreList>,

    /// The location of the graph's legend.
    #[serde(default)]
    pub(crate) legend_position: Option<String>,

    /// An upper temperature value for the graph; entries higher than this will be hidden. If not set,
    /// there is no limit.
    ///
    /// Is in the unit of `temperature_type`.
    #[serde(default)]
    pub(crate) max_temp: Option<f64>,
}
