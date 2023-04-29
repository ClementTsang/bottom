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
}
