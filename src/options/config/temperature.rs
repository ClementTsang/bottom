use serde::Deserialize;

use super::IgnoreList;

/// Temperature configuration.
#[derive(Clone, Debug, Default, Deserialize)]
#[cfg_attr(feature = "generate_schema", derive(schemars::JsonSchema))]
pub struct TempConfig {
    /// A filter over the sensor names.
    pub sensor_filter: Option<IgnoreList>,
}
