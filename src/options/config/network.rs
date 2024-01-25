use serde::Deserialize;

#[derive(Clone, Debug, Default, Deserialize)]
pub(crate) struct NetworkConfig {
    #[serde(default)]
    pub(crate) network_use_bytes: bool,
    #[serde(default)]
    pub(crate) network_use_log: bool,
    #[serde(default)]
    pub(crate) network_use_binary_prefix: bool,
    #[serde(default)]
    pub(crate) use_old_network_legend: bool,
}
