use serde::Deserialize;

#[derive(Clone, Debug, Default, Deserialize)]
pub(crate) struct NetworkConfig {
    pub(crate) network_use_bytes: Option<bool>,
    pub(crate) network_use_log: Option<bool>,
    pub(crate) network_use_binary_prefix: Option<bool>,
    pub(crate) use_old_network_legend: Option<bool>,
}
