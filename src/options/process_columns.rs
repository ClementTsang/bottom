use serde::Deserialize;

use crate::widgets::ProcWidgetColumn;

/// Process column settings.
#[derive(Clone, Debug, Default, Deserialize)]
pub struct ProcessConfig {
    pub columns: Option<Vec<ProcWidgetColumn>>,
}

#[cfg(test)]
mod test {
    use crate::widgets::ProcWidgetColumn;

    use super::ProcessConfig;

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
            generated.columns,
            Some(vec![
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
            ]),
        );
    }

    #[test]
    fn process_column_settings_2() {
        let config = r#"columns = ["MEM", "TWrite", "Cpuz", "read", "wps"]"#;
        toml_edit::de::from_str::<ProcessConfig>(config).expect_err("Should error out!");
    }

    #[test]
    fn process_column_settings_3() {
        let config = r#"columns = ["Twrite", "T.Write"]"#;
        let generated: ProcessConfig = toml_edit::de::from_str(config).unwrap();
        assert_eq!(
            generated.columns,
            Some(vec![ProcWidgetColumn::TotalWrite; 2])
        );

        let config = r#"columns = ["Tread", "T.read"]"#;
        let generated: ProcessConfig = toml_edit::de::from_str(config).unwrap();
        assert_eq!(
            generated.columns,
            Some(vec![ProcWidgetColumn::TotalRead; 2])
        );

        let config = r#"columns = ["read", "rps", "r/s"]"#;
        let generated: ProcessConfig = toml_edit::de::from_str(config).unwrap();
        assert_eq!(
            generated.columns,
            Some(vec![ProcWidgetColumn::ReadPerSecond; 3])
        );

        let config = r#"columns = ["write", "wps", "w/s"]"#;
        let generated: ProcessConfig = toml_edit::de::from_str(config).unwrap();
        assert_eq!(
            generated.columns,
            Some(vec![ProcWidgetColumn::WritePerSecond; 3])
        );
    }
}
