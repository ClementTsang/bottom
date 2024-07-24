use serde::{Deserialize, Serialize};

use super::Color;

/// Styling specific to the battery widget.
#[derive(Clone, Debug, Default, Deserialize, Serialize)]
#[cfg_attr(feature = "generate_schema", derive(schemars::JsonSchema))]
pub(crate) struct BatteryStyle {
    pub(crate) high_battery_color: Option<Color>,
    pub(crate) medium_battery_color: Option<Color>,
    pub(crate) low_battery_color: Option<Color>,
}
