use indoc::indoc;
use serde::Deserialize;

use crate::args::CpuArgs;

use super::DefaultConfig;

/// The default selection of the CPU widget. If the given selection is invalid, we will fall back to all.
#[derive(Clone, Copy, Debug, Default, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum CpuDefault {
    #[default]
    All,
    #[serde(alias = "avg")]
    Average,
}

/// Process column settings.

#[derive(Clone, Debug, Default, Deserialize)]
pub(crate) struct CpuConfig {
    #[serde(flatten)]
    pub(crate) args: CpuArgs,
    #[serde(default)]
    pub(crate) default: CpuDefault,
}

impl DefaultConfig for CpuConfig {
    fn default_config() -> String {
        let s = indoc! {r##"
        
        "##};

        s.to_string()
    }
}

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn default_cpu_default() {
        let config = "";
        let generated: CpuConfig = toml_edit::de::from_str(config).unwrap();
        match generated.default {
            CpuDefault::All => {}
            CpuDefault::Average => {
                panic!("the default should be all")
            }
        }
    }

    #[test]
    fn all_cpu_default() {
        let config = r#"
            default = "all"
        "#;
        let generated: CpuConfig = toml_edit::de::from_str(config).unwrap();
        match generated.default {
            CpuDefault::All => {}
            CpuDefault::Average => {
                panic!("the default should be all")
            }
        }
    }

    #[test]
    fn avg_cpu_default() {
        let config = r#"
            default = "avg"
        "#;

        let generated: CpuConfig = toml_edit::de::from_str(config).unwrap();
        match generated.default {
            CpuDefault::All => {
                panic!("the avg should be set")
            }
            CpuDefault::Average => {}
        }
    }

    #[test]
    fn average_cpu_default() {
        let config = r#"
            default = "average"
        "#;

        let generated: CpuConfig = toml_edit::de::from_str(config).unwrap();
        match generated.default {
            CpuDefault::All => {
                panic!("the avg should be set")
            }
            CpuDefault::Average => {}
        }
    }
}
