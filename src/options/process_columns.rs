use serde::{Deserialize, Serialize};

use crate::widgets::ProcColumn;

/// Process column settings.
#[derive(Clone, Debug, Default, Deserialize, Serialize)]
pub struct ProcessConfig {
    pub columns: Option<Vec<ProcColumn>>,
}

#[cfg(test)]
mod test {
    use crate::widgets::ProcColumn;

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
            columns = ["CPU%", "PiD", "user", "MEM", "Tread", "T.Write", "Rps", "W/s"]
        "#;

        let generated: ProcessConfig = toml_edit::de::from_str(config).unwrap();
        assert_eq!(
            generated.columns,
            Some(vec![
                ProcColumn::CpuPercent,
                ProcColumn::Pid,
                ProcColumn::User,
                ProcColumn::MemoryVal,
                ProcColumn::TotalRead,
                ProcColumn::TotalWrite,
                ProcColumn::ReadPerSecond,
                ProcColumn::WritePerSecond,
            ]),
        );
    }

    #[test]
    fn process_column_settings_2() {
        let config = r#"
            columns = ["MEM%"]
        "#;

        let generated: ProcessConfig = toml_edit::de::from_str(config).unwrap();
        assert_eq!(generated.columns, Some(vec![ProcColumn::MemoryPercent]));
    }

    #[test]
    fn process_column_settings_3() {
        let config = r#"
            columns = ["MEM%", "TWrite", "Cpuz", "read", "wps"]
        "#;

        toml_edit::de::from_str::<ProcessConfig>(config).expect_err("Should error out!");
    }

    #[test]
    fn process_column_settings_4() {
        let config = r#"columns = ["Twrite", "T.Write"]"#;
        let generated: ProcessConfig = toml_edit::de::from_str(config).unwrap();
        assert_eq!(generated.columns, Some(vec![ProcColumn::TotalWrite; 2]));

        let config = r#"columns = ["Tread", "T.read"]"#;
        let generated: ProcessConfig = toml_edit::de::from_str(config).unwrap();
        assert_eq!(generated.columns, Some(vec![ProcColumn::TotalRead; 2]));

        let config = r#"columns = ["read", "rps", "r/s"]"#;
        let generated: ProcessConfig = toml_edit::de::from_str(config).unwrap();
        assert_eq!(generated.columns, Some(vec![ProcColumn::ReadPerSecond; 3]));

        let config = r#"columns = ["write", "wps", "w/s"]"#;
        let generated: ProcessConfig = toml_edit::de::from_str(config).unwrap();
        assert_eq!(generated.columns, Some(vec![ProcColumn::WritePerSecond; 3]));
    }
}
