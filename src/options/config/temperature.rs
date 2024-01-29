use serde::Deserialize;

use crate::args::TemperatureArgs;

#[derive(Clone, Debug, Default, Deserialize)]
pub(crate) struct TemperatureConfig {
    #[serde(flatten)]
    pub(crate) args: TemperatureArgs,
    pub(crate) temperature_type: Option<String>,
}
