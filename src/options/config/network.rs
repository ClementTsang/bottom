use serde::Deserialize;

use crate::args::NetworkArgs;

use super::DefaultConfig;

#[derive(Clone, Debug, Default, Deserialize)]
pub(crate) struct NetworkConfig {
    #[serde(flatten)]
    pub(crate) args: NetworkArgs,
}

impl DefaultConfig for NetworkConfig {
    fn default_config() -> String {
        todo!()
    }
}
