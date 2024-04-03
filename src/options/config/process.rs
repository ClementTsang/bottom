use indoc::indoc;
use serde::Deserialize;

use crate::{args::ProcessArgs, widgets::ProcWidgetColumn};

use super::DefaultConfig;

/// Process column settings.
#[derive(Clone, Debug, Default, Deserialize)]
pub(crate) struct ProcessConfig {
    #[serde(flatten)]
    pub(crate) args: ProcessArgs,
    pub(crate) columns: Option<Vec<ProcWidgetColumn>>,
}

impl DefaultConfig for ProcessConfig {
    fn default_config() -> String {
        let s = indoc! {r##"
            # Enables case sensitivity by default when searching for a process.
            # case_sensitive = false

            # Calculates process CPU usage as a percentage of current usage rather than total usage.
            # current_usage = false

            # Hides advanced process stopping options on Unix-like systems. Signal 15 (TERM) will be sent when stopping a process.
            # disable_advanced_kill = false

            # Groups processes with the same name by default.
            # group_processes = false

            # Defaults to showing process memory usage by value. Otherwise, it defaults to showing it by percentage.
            # mem_as_value = false

            # Shows the full command name instead of just the process name by default.
            # process_command = false

            # Enables regex by default while searching.
            # regex = false

            # Makes the process widget use tree mode by default.
            # tree = false

            # Show process CPU% usage without averaging over the number of CPU cores.
            # unnormalized_cpu = false

            # Enables whole-word matching by default while searching.
            # whole_word = false
        "##};

        s.to_string()
    }
}

#[cfg(test)]
mod test {
    use super::ProcessConfig;
    use crate::widgets::ProcWidgetColumn;

    #[test]
    fn empty_column_setting() {
        let config = "";
        let generated: ProcessConfig = toml_edit::de::from_str(config).unwrap();
        assert!(generated.columns.is_none());
    }

    #[test]
    fn process_column_settings() {
        let config = r#"
            columns = ["CPU%", "PiD", "user", "MEM", "Tread", "T.Write", "Rps", "W/s", "tiMe", "USER", "state"]
        "#;

        let generated: ProcessConfig = toml_edit::de::from_str(config).unwrap();
        assert_eq!(
            generated.columns.unwrap(),
            vec![
                ProcWidgetColumn::Cpu,
                ProcWidgetColumn::PidOrCount,
                ProcWidgetColumn::User,
                ProcWidgetColumn::Mem,
                ProcWidgetColumn::TotalRead,
                ProcWidgetColumn::TotalWrite,
                ProcWidgetColumn::ReadPerSecond,
                ProcWidgetColumn::WritePerSecond,
                ProcWidgetColumn::Time,
                ProcWidgetColumn::User,
                ProcWidgetColumn::State,
            ],
        );
    }

    #[test]
    fn process_column_settings_2() {
        let config = r#"columns = ["MEM", "TWrite", "fake", "read", "wps"]"#;
        toml_edit::de::from_str::<ProcessConfig>(config).expect_err("Should error out!");
    }

    #[test]
    fn process_column_settings_3() {
        let config = r#"columns = ["Twrite", "T.Write"]"#;
        let generated: ProcessConfig = toml_edit::de::from_str(config).unwrap();
        let columns = generated.columns.unwrap();
        assert_eq!(columns, vec![ProcWidgetColumn::TotalWrite; 2]);

        let config = r#"columns = ["Tread", "T.read"]"#;
        let generated: ProcessConfig = toml_edit::de::from_str(config).unwrap();
        let columns = generated.columns.unwrap();
        assert_eq!(columns, vec![ProcWidgetColumn::TotalRead; 2]);

        let config = r#"columns = ["read", "rps", "r/s"]"#;
        let generated: ProcessConfig = toml_edit::de::from_str(config).unwrap();
        let columns = generated.columns.unwrap();
        assert_eq!(columns, vec![ProcWidgetColumn::ReadPerSecond; 3]);

        let config = r#"columns = ["write", "wps", "w/s"]"#;
        let generated: ProcessConfig = toml_edit::de::from_str(config).unwrap();
        let columns = generated.columns.unwrap();
        assert_eq!(columns, vec![ProcWidgetColumn::WritePerSecond; 3]);
    }
}
