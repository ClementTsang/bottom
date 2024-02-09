use serde::Deserialize;

use crate::args::BatteryArgs;

use super::DefaultConfig;

#[derive(Clone, Debug, Default, Deserialize)]
pub(crate) struct BatteryConfig {
    pub(crate) args: BatteryArgs,
}

impl DefaultConfig for BatteryConfig {
    fn default_config() -> String {
        todo!()
    }
}
