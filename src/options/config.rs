pub mod cpu;
pub mod disk;
pub mod flags;
mod ignore_list;
pub mod layout;
pub mod network;
pub mod process;
pub mod style;
pub mod temperature;

use disk::DiskConfig;
use flags::GeneralConfig;
use network::NetworkConfig;
use serde::{Deserialize, Serialize};
use style::StyleConfig;
use temperature::TempConfig;

pub use self::ignore_list::IgnoreList;
use self::{cpu::CpuConfig, layout::Row, process::ProcessesConfig};

/// Overall config for `bottom`.
#[derive(Clone, Debug, Default, Deserialize)]
#[cfg_attr(feature = "generate_schema", derive(schemars::JsonSchema))]
#[cfg_attr(test, serde(deny_unknown_fields), derive(PartialEq, Eq))]
pub struct Config {
    pub(crate) flags: Option<GeneralConfig>,
    pub(crate) styles: Option<StyleConfig>,
    pub(crate) row: Option<Vec<Row>>,
    pub(crate) processes: Option<ProcessesConfig>,
    pub(crate) disk: Option<DiskConfig>,
    pub(crate) temperature: Option<TempConfig>,
    pub(crate) network: Option<NetworkConfig>,
    pub(crate) cpu: Option<CpuConfig>,
}

#[derive(Clone, Debug, Deserialize, Serialize)]
#[serde(untagged)]
#[cfg_attr(feature = "generate_schema", derive(schemars::JsonSchema))]
#[cfg_attr(test, derive(PartialEq, Eq))]
pub(crate) enum StringOrNum {
    String(String),
    Num(u64),
}

impl From<String> for StringOrNum {
    fn from(value: String) -> Self {
        StringOrNum::String(value)
    }
}

impl From<u64> for StringOrNum {
    fn from(value: u64) -> Self {
        StringOrNum::Num(value)
    }
}

#[cfg(test)]
mod test {

    // Test all valid configs in the integration test folder and ensure they are accepted.
    // We need this separated as only test library code sets `serde(deny_unknown_fields)`.
    #[test]
    #[cfg(feature = "default")]
    fn test_integration_valid_configs() {
        use std::fs;

        use super::Config;

        for config_path in fs::read_dir("./tests/valid_configs").unwrap() {
            let dir_entry = config_path.unwrap();
            let path = dir_entry.path();

            if path.is_file() {
                let config_path_str = path.display().to_string();
                let config_str = fs::read_to_string(path).unwrap();

                toml_edit::de::from_str::<Config>(&config_str)
                    .unwrap_or_else(|_| panic!("incorrectly rejected '{config_path_str}'"));
            }
        }
    }

    // I didn't do an invalid config test as a lot of them _are_ valid Config when parsed,
    // but fail other checks.
}
