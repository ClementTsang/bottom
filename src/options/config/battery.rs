use serde::Deserialize;

use crate::args::BatteryArgs;

#[derive(Clone, Debug, Default, Deserialize)]
pub(crate) struct BatteryConfig {
    pub(crate) args: BatteryArgs,
}
