use serde::{Deserialize, Serialize};

use super::ColorStr;

/// Styling specific to the battery widget.
#[derive(Clone, Debug, Default, Deserialize, Serialize)]
#[cfg_attr(feature = "generate_schema", derive(schemars::JsonSchema))]
pub(crate) struct BatteryStyle {
    pub(crate) high_battery: Option<ColorStr>,
    pub(crate) medium_battery: Option<ColorStr>,
    pub(crate) low_battery: Option<ColorStr>,
}
