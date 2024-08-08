use serde::{Deserialize, Serialize};

use super::ColorStr;

/// Styling specific to the battery widget.
#[derive(Clone, Debug, Default, Deserialize, Serialize)]
#[cfg_attr(feature = "generate_schema", derive(schemars::JsonSchema))]
#[cfg_attr(test, serde(deny_unknown_fields), derive(PartialEq, Eq))]
pub(crate) struct BatteryStyle {
    /// The colour of the battery widget bar when the battery is over 50%.
    #[serde(alias = "high_battery_colour")]
    pub(crate) high_battery_color: Option<ColorStr>,

    /// The colour of the battery widget bar when the battery between 10% to 50%.
    #[serde(alias = "medium_battery_colour")]
    pub(crate) medium_battery_color: Option<ColorStr>,

    /// The colour of the battery widget bar when the battery is under 10%.
    #[serde(alias = "low_battery_colour")]
    pub(crate) low_battery_color: Option<ColorStr>,
}
