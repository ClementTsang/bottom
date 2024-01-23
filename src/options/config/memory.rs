use serde::Deserialize;

#[derive(Clone, Debug, Default, Deserialize)]
pub(crate) struct MemoryConfig {
    pub(crate) mem_as_value: Option<bool>,
    pub(crate) enable_cache_memory: Option<bool>,
}
