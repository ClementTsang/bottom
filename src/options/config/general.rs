use clap::ArgMatches;
use serde::Deserialize;

use crate::args::GeneralArgs;

#[derive(Clone, Debug, Default, Deserialize)]
pub(crate) struct GeneralConfig {
    #[serde(flatten)]
    pub(crate) args: GeneralArgs,
}
