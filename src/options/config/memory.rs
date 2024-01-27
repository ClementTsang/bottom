use serde::Deserialize;

use crate::args::MemoryArgs;

#[derive(Clone, Debug, Default, Deserialize)]
pub(crate) struct MemoryConfig {
    #[serde(flatten)]
    pub(crate) args: MemoryArgs,
}
