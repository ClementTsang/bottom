pub mod colours;
pub mod palettes;

use serde::Deserialize;

use crate::args::StyleArgs;

use super::DefaultConfig;

#[derive(Clone, Debug, Default, Deserialize)]
pub(crate) struct StyleConfig {
    #[serde(flatten)]
    pub(crate) args: StyleArgs,
    // TODO: Maybe also put colours here?
}

impl DefaultConfig for StyleConfig {
    fn default_config() -> String {
        todo!()
    }
}
