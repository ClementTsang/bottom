use serde::Deserialize;

use super::IgnoreList;

/// Temperature configuration.
#[derive(Clone, Debug, Default, Deserialize)]
#[cfg_attr(feature = "generate_schema", derive(schemars::JsonSchema))]
#[cfg_attr(test, serde(deny_unknown_fields), derive(PartialEq, Eq))]
pub(crate) struct TempConfig {
    /// A filter over the sensor names.
    pub(crate) sensor_filter: Option<IgnoreList>,
}
