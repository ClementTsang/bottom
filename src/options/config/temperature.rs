use serde::Deserialize;

use crate::args::TemperatureArgs;

use super::DefaultConfig;

#[derive(Clone, Debug, Default, Deserialize)]
pub(crate) struct TemperatureConfig {
    #[serde(flatten)]
    pub(crate) args: TemperatureArgs,
    pub(crate) temperature_type: Option<String>,
}

impl DefaultConfig for TemperatureConfig {
    fn default_config() -> String {
        todo!()
    }
}
