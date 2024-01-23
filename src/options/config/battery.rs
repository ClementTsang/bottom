use serde::Deserialize;

#[derive(Clone, Debug, Default, Deserialize)]
pub(crate) struct BatteryOptions {
    pub(crate) battery: Option<bool>,
}
