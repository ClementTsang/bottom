use indoc::indoc;
use serde::Deserialize;

use crate::args::CpuArgs;

use super::DefaultConfig;

/// Process column settings.

#[derive(Clone, Debug, Default, Deserialize)]
pub(crate) struct CpuConfig {
    #[serde(flatten)]
    pub(crate) args: CpuArgs,
}

impl DefaultConfig for CpuConfig {
    fn default_config() -> String {
        let s = indoc! {r##"
            # Sets which CPU entry is selected by default.
            # default_cpu_entry = "all"

            # Hides the average CPU usage entry from being shown.
            # hide_avg_cpu = false

            # Puts the CPU chart legend on the left side.
            # left_legend = false
        "##};

        s.to_string()
    }
}

#[cfg(test)]
mod test {
    use crate::args::CpuDefault;

    use super::*;

    #[test]
    fn default_cpu_default() {
        let config = "";
        let generated: CpuConfig = toml_edit::de::from_str(config).unwrap();
        match generated.args.default_cpu_entry {
            CpuDefault::All => {}
            CpuDefault::Average => {
                panic!("the default should be all")
            }
        }
    }

    #[test]
    fn all_cpu_default() {
        let config = r#"
            default_cpu_entry = "all"
        "#;
        let generated: CpuConfig = toml_edit::de::from_str(config).unwrap();
        match generated.args.default_cpu_entry {
            CpuDefault::All => {}
            CpuDefault::Average => {
                panic!("the default should be all")
            }
        }
    }

    #[test]
    fn avg_cpu_default() {
        let config = r#"
            default_cpu_entry = "avg"
        "#;

        let generated: CpuConfig = toml_edit::de::from_str(config).unwrap();
        match generated.args.default_cpu_entry {
            CpuDefault::All => {
                panic!("the avg should be set")
            }
            CpuDefault::Average => {}
        }
    }

    #[test]
    fn average_cpu_default() {
        let config = r#"
            default_cpu_entry = "average"
        "#;

        let generated: CpuConfig = toml_edit::de::from_str(config).unwrap();
        match generated.args.default_cpu_entry {
            CpuDefault::All => {
                panic!("the avg should be set")
            }
            CpuDefault::Average => {}
        }
    }
}
