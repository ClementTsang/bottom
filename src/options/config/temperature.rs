use serde::Deserialize;

#[derive(Clone, Debug, Default, Deserialize)]
pub(crate) struct TemperatureConfig {
    pub(crate) temperature_type: Option<String>,
}
