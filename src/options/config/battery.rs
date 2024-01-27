use serde::Deserialize;

use crate::args::BatteryArgs;

#[cfg(feature = "battery")]
#[derive(Clone, Debug, Default, Deserialize)]
pub(crate) struct BatteryOptions {
    pub(crate) args: BatteryArgs,
}
