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
    fn set_column_setting() {
        let config = r#"
            columns = ["CPU%", "PiD", "user", "MEM"]
        "#;

        let generated: ProcessConfig = toml_edit::de::from_str(config).unwrap();
        assert_eq!(
            generated.columns,
            Some(vec![
                ProcColumn::CpuPercent,
                ProcColumn::Pid,
                ProcColumn::User,
                ProcColumn::MemoryVal
            ]),
        );
    }
}
