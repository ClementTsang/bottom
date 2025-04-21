use serde::Deserialize;

/// The default selection of the CPU widget. If the given selection is invalid,
/// we will fall back to all.
#[derive(Clone, Copy, Debug, Default, Deserialize)]
#[cfg_attr(feature = "generate_schema", derive(schemars::JsonSchema))]
#[serde(rename_all = "lowercase")]
#[cfg_attr(test, derive(PartialEq, Eq))]
pub(crate) enum CpuDefault {
    #[default]
    All,
    #[serde(alias = "avg")]
    Average,
}

/// CPU column settings.
#[derive(Clone, Debug, Default, Deserialize)]
#[cfg_attr(feature = "generate_schema", derive(schemars::JsonSchema))]
#[cfg_attr(test, serde(deny_unknown_fields), derive(PartialEq, Eq))]
pub(crate) struct CpuConfig {
    #[serde(default)]
    pub(crate) default: CpuDefault,
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
