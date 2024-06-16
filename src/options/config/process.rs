use serde::Deserialize;

use crate::widgets::ProcWidgetColumn;

/// Process configuration.
#[derive(Clone, Debug, Default, Deserialize)]
pub struct ProcessesConfig {
    /// A list of process widget columns.
    #[serde(default)]
    pub columns: Vec<ProcWidgetColumn>,
}

#[cfg(test)]
mod test {
    use super::ProcessesConfig;
    use crate::widgets::ProcWidgetColumn;

    #[test]
    fn empty_column_setting() {
        let config = "";
        let generated: ProcessesConfig = toml_edit::de::from_str(config).unwrap();
        assert!(generated.columns.is_empty());
    }

    #[test]
    fn process_column_settings() {
        let config = r#"
            columns = ["CPU%", "PiD", "user", "MEM", "Tread", "T.Write", "Rps", "W/s", "tiMe", "USER", "state"]
        "#;

        let generated: ProcessesConfig = toml_edit::de::from_str(config).unwrap();
        assert_eq!(
            generated.columns,
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
        let config = r#"columns = ["MEM", "TWrite", "Cpuz", "read", "wps"]"#;
        toml_edit::de::from_str::<ProcessesConfig>(config).expect_err("Should error out!");
    }

    #[test]
    fn process_column_settings_3() {
        let config = r#"columns = ["Twrite", "T.Write"]"#;
        let generated: ProcessesConfig = toml_edit::de::from_str(config).unwrap();
        assert_eq!(generated.columns, vec![ProcWidgetColumn::TotalWrite; 2]);

        let config = r#"columns = ["Tread", "T.read"]"#;
        let generated: ProcessesConfig = toml_edit::de::from_str(config).unwrap();
        assert_eq!(generated.columns, vec![ProcWidgetColumn::TotalRead; 2]);

        let config = r#"columns = ["read", "rps", "r/s"]"#;
        let generated: ProcessesConfig = toml_edit::de::from_str(config).unwrap();
        assert_eq!(generated.columns, vec![ProcWidgetColumn::ReadPerSecond; 3]);

        let config = r#"columns = ["write", "wps", "w/s"]"#;
        let generated: ProcessesConfig = toml_edit::de::from_str(config).unwrap();
        assert_eq!(generated.columns, vec![ProcWidgetColumn::WritePerSecond; 3]);
    }
}
