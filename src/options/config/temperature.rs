use indoc::indoc;
use serde::Deserialize;

use crate::args::TemperatureArgs;

use super::DefaultConfig;

#[derive(Clone, Debug, Default, Deserialize)]
pub(crate) struct TemperatureConfig {
    #[serde(flatten)]
    pub(crate) args: TemperatureArgs,
    pub(crate) temperature_type: Option<String>,
}

impl DefaultConfig for TemperatureConfig {
    fn default_config() -> String {
        let s = indoc! {r##"
            # The temperature unit. Supported values are "[c]elsius", "[f]ahrenheit", and "[k]elvin".
            # temperature_type = "celsius"
        "##};

        s.to_string()
    }
}
