use serde::Deserialize;

use crate::args::MemoryArgs;

use super::DefaultConfig;

#[derive(Clone, Debug, Default, Deserialize)]
pub(crate) struct MemoryConfig {
    #[serde(flatten)]
    pub(crate) args: MemoryArgs,
}

impl DefaultConfig for MemoryConfig {
    fn default_config() -> String {
        todo!()
    }
}
