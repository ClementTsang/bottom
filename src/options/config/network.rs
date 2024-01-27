use serde::Deserialize;

use crate::args::NetworkArgs;

#[derive(Clone, Debug, Default, Deserialize)]
pub(crate) struct NetworkConfig {
    #[serde(flatten)]
    pub(crate) args: NetworkArgs,
}
