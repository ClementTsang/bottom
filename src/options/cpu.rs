use serde::Deserialize;

/// The default selection of the CPU widget. If the given selection is invalid, we will fall back to all.
#[derive(Clone, Copy, Debug, Default, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum CpuDefault {
    #[default]
    All,
    #[serde(alias = "avg")]
    Average,
}

/// Process column settings.
#[derive(Clone, Debug, Default, Deserialize)]
pub struct CpuConfig {
    #[serde(default)]
    pub default: CpuDefault,
}
