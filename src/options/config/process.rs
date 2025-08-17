use serde::Deserialize;

use crate::widgets::ProcColumn;

/// Process configuration.
#[derive(Clone, Debug, Default, Deserialize)]
#[cfg_attr(feature = "generate_schema", derive(schemars::JsonSchema))]
#[cfg_attr(test, serde(deny_unknown_fields), derive(PartialEq, Eq))]
pub(crate) struct ProcessesConfig {
    /// A list of process widget columns.
    #[serde(default)]
    pub columns: Vec<ProcColumn>, // TODO: make this more composable(?) in the future, we might need to rethink how it's done for custom widgets

    /// Whether to get process child threads.
    pub get_threads: Option<bool>,
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
        let config = r#"
            columns = ["CPU%", "PiD", "user", "MEM", "virt", "Tread", "T.Write", "Rps", "W/s", "tiMe", "USER", "state"]
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
            ],
        );
    }

    #[test]
    fn bad_process_column_config() {
        let config = r#"columns = ["MEM", "TWrite", "Cpuz", "read", "wps"]"#;
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
