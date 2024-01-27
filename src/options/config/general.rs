use clap::ArgMatches;
use serde::Deserialize;

use crate::args::GeneralArgs;

#[derive(Clone, Debug, Default, Deserialize)]
pub(crate) struct GeneralConfig {
    #[serde(flatten)]
    pub(crate) args: GeneralArgs,
}

impl GeneralConfig {
    pub(crate) fn merge_with_args(&mut self, args: &ArgMatches) {}
}
