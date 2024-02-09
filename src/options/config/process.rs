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
