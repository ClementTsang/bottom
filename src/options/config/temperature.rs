use serde::Deserialize;

use super::IgnoreList;
use crate::widgets::TempWidgetColumn;

/// Temperature configuration.
#[derive(Clone, Debug, Default, Deserialize)]
#[cfg_attr(feature = "generate_schema", derive(schemars::JsonSchema))]
#[cfg_attr(test, serde(deny_unknown_fields), derive(PartialEq, Eq))]
pub(crate) struct TempConfig {
    /// A filter over the sensor names.
    pub(crate) sensor_filter: Option<IgnoreList>,

    /// The default sort column.
    #[serde(default)]
    pub(crate) default_sort: Option<TempWidgetColumn>,

    /// The temperature unit to use. One of "celsius", "fahrenheit", or "kelvin".
    pub(crate) temperature_type: Option<String>,
}
