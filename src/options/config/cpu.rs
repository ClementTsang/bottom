use serde::Deserialize;

/// The default selected entry of the CPU widget.
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
    /// The default selected entry of the CPU widget.
    #[serde(default)]
    pub(crate) default: CpuDefault,

    /// Whether to show a decimal place for CPU usage values.
    pub(crate) show_decimal: Option<bool>,

    /// Whether to hide the average CPU entry.
    pub(crate) hide_avg_cpu: Option<bool>,

    /// Whether to put the CPU chart legend on the left side.
    pub(crate) left_legend: Option<bool>,

    /// Whether to give the average CPU entry a dedicated row in basic mode.
    pub(crate) basic_average_cpu_row: Option<bool>,
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
