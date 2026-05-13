use serde::Deserialize;

use crate::widgets::ProcColumn;

/// Process configuration fields.
#[derive(Clone, Debug, Default, Deserialize)]
#[cfg_attr(feature = "generate_schema", derive(schemars::JsonSchema))]
#[cfg_attr(test, serde(deny_unknown_fields), derive(PartialEq, Eq))]
pub(crate) struct ProcessesConfig {
    /// A list of process widget columns.
    ///
    // TODO: make this more composable(?) in the future, we might need to
    // rethink how it's done for custom widgets
    #[serde(default)]
    pub columns: Vec<ProcColumn>,

    /// The default sort column.
    #[serde(default)]
    pub default_sort: Option<ProcColumn>,

    /// Whether to get process child threads.
    pub get_threads: Option<bool>,

    /// Hide kernel threads from being shown.
    pub hide_k_threads: Option<bool>,

    /// Collapse the process tree by default when tree mode is set.
    pub tree_collapse: Option<bool>,

    /// Shows the full command name instead of the process name by default.
    pub process_command: Option<bool>,

    // This does nothing on Windows, but we leave it enabled to make the config file consistent
    // across platforms.
    //
    // #[cfg(any(target_os = "linux", target_os = "macos", target_os = "freebsd"))]
    /// Disable the advanced kill dialog and just show the basic one with no options.
    pub disable_advanced_kill: Option<bool>,

    /// Defaults to showing process memory usage by value.
    pub default_memory_value: Option<bool>,

    /// Groups processes with the same name by default. No effect if `--tree` is set.
    pub default_grouped: Option<bool>,

    /// Enables regex by default while searching.
    pub regex: Option<bool>,

    /// Enables case sensitivity by default when searching.
    pub case_sensitive: Option<bool>,

    /// Enables whole-word matching by default while searching.
    pub whole_word: Option<bool>,

    /// Makes the process widget use tree mode by default.
    pub default_tree: Option<bool>,

    /// Calculates process CPU usage as a percentage of current usage rather than total usage.
    pub current_usage: Option<bool>,

    /// Show process CPU% usage without averaging over the number of CPU cores.
    pub unnormalized_cpu: Option<bool>,
}

#[cfg(test)]
mod test {
    use super::{ProcColumn, ProcessesConfig};
    use crate::widgets::ProcWidgetColumn;

    #[test]
    fn empty_column_setting() {
        let config = "";
        let generated: ProcessesConfig = toml_edit::de::from_str(config).unwrap();
        assert!(generated.columns.is_empty());
    }

    fn to_columns(columns: Vec<ProcColumn>) -> Vec<ProcWidgetColumn> {
        columns
            .iter()
            .map(ProcWidgetColumn::from)
            .collect::<Vec<_>>()
    }

    #[test]
    fn valid_process_column_config() {
        #[cfg(unix)]
        let config = r#"
            columns = ["CPU%", "PiD", "user", "MEM", "virt", "Tread", "T.Write", "Rps", "W/s", "tiMe", "USER", "state", "prioRity", "Nice"]
        "#;

        #[cfg(target_os = "windows")]
        let config = r#"
            columns = ["CPU%", "PiD", "user", "MEM", "virt", "Tread", "T.Write", "Rps", "W/s", "tiMe", "USER", "state", "prioRity"]
        "#;

        let generated: ProcessesConfig = toml_edit::de::from_str(config).unwrap();
        assert_eq!(
            to_columns(generated.columns),
            vec![
                ProcWidgetColumn::Cpu,
                ProcWidgetColumn::PidOrCount,
                ProcWidgetColumn::User,
                ProcWidgetColumn::Mem,
                ProcWidgetColumn::VirtualMem,
                ProcWidgetColumn::TotalRead,
                ProcWidgetColumn::TotalWrite,
                ProcWidgetColumn::ReadPerSecond,
                ProcWidgetColumn::WritePerSecond,
                ProcWidgetColumn::Time,
                ProcWidgetColumn::User,
                ProcWidgetColumn::State,
                ProcWidgetColumn::Priority,
                #[cfg(unix)]
                ProcWidgetColumn::Nice,
            ],
        );
    }

    #[test]
    fn bad_process_column_config() {
        let config = r#"columns = ["MEM", "TWrite", "Cpuz", "read", "wps"]"#;
        toml_edit::de::from_str::<ProcessesConfig>(config).expect_err("Should error out!");
    }

    #[test]
    fn valid_default_sort_config() {
        let config = r#"default_sort = "mem""#;
        let generated: ProcessesConfig = toml_edit::de::from_str(config).unwrap();
        assert_eq!(generated.default_sort, Some(ProcColumn::MemPercent));

        let config = r#"default_sort = "PID""#;
        let generated: ProcessesConfig = toml_edit::de::from_str(config).unwrap();
        assert_eq!(generated.default_sort, Some(ProcColumn::Pid));

        let config = "";
        let generated: ProcessesConfig = toml_edit::de::from_str(config).unwrap();
        assert_eq!(generated.default_sort, None);
    }

    #[test]
    fn invalid_default_sort_config() {
        let config = r#"default_sort = "soup""#;
        toml_edit::de::from_str::<ProcessesConfig>(config).expect_err("Should error out!");
    }

    #[test]
    fn valid_process_column_config_2() {
        let config = r#"columns = ["Twrite", "T.Write"]"#;
        let generated: ProcessesConfig = toml_edit::de::from_str(config).unwrap();
        assert_eq!(
            to_columns(generated.columns),
            vec![ProcWidgetColumn::TotalWrite; 2]
        );

        let config = r#"columns = ["Tread", "T.read"]"#;
        let generated: ProcessesConfig = toml_edit::de::from_str(config).unwrap();
        assert_eq!(
            to_columns(generated.columns),
            vec![ProcWidgetColumn::TotalRead; 2]
        );

        let config = r#"columns = ["read", "rps", "r/s"]"#;
        let generated: ProcessesConfig = toml_edit::de::from_str(config).unwrap();
        assert_eq!(
            to_columns(generated.columns),
            vec![ProcWidgetColumn::ReadPerSecond; 3]
        );

        let config = r#"columns = ["write", "wps", "w/s"]"#;
        let generated: ProcessesConfig = toml_edit::de::from_str(config).unwrap();
        assert_eq!(
            to_columns(generated.columns),
            vec![ProcWidgetColumn::WritePerSecond; 3]
        );
    }
}
