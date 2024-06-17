use serde::Deserialize;

use crate::widgets::ProcWidgetColumn;

/// Process configuration.
#[derive(Clone, Debug, Default, Deserialize)]
#[cfg_attr(feature = "generate_schema", derive(schemars::JsonSchema))]
pub struct ProcessesConfig {
    /// A list of process widget columns.
    #[serde(default)]
    pub columns: Vec<ProcColumn>,
}

/// A column in the process widget.
#[derive(Clone, Debug)]
#[cfg_attr(
    feature = "generate_schema",
    derive(schemars::JsonSchema, strum::VariantArray)
)]
pub enum ProcColumn {
    Pid,
    Count,
    Name,
    Command,
    CpuPercent,
    Mem,
    MemPercent,
    Read,
    Write,
    TotalRead,
    TotalWrite,
    State,
    User,
    Time,
    #[cfg(feature = "gpu")]
    GpuMem,
    #[cfg(feature = "gpu")]
    GpuPercent,
}

impl ProcColumn {
    /// An ugly hack to generate the JSON schema.
    #[cfg(feature = "generate_schema")]
    pub fn get_schema_names(&self) -> &[&'static str] {
        match self {
            ProcColumn::Pid => &["PID"],
            ProcColumn::Count => &["Count"],
            ProcColumn::Name => &["Name"],
            ProcColumn::Command => &["Command"],
            ProcColumn::CpuPercent => &["CPU%"],
            ProcColumn::Mem => &["Mem"],
            ProcColumn::MemPercent => &["Mem%"],
            ProcColumn::Read => &["R/s", "Read", "Rps"],
            ProcColumn::Write => &["W/s", "Write", "Wps"],
            ProcColumn::TotalRead => &["T.Read", "TWrite"],
            ProcColumn::TotalWrite => &["T.Write", "TRead"],
            ProcColumn::State => &["State"],
            ProcColumn::User => &["User"],
            ProcColumn::Time => &["Time"],
            ProcColumn::GpuMem => &["GMem", "GMem%"],
            ProcColumn::GpuPercent => &["GPU%"],
        }
    }
}

impl<'de> Deserialize<'de> for ProcColumn {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        let value = String::deserialize(deserializer)?.to_lowercase();
        match value.as_str() {
            "cpu%" => Ok(ProcColumn::CpuPercent),
            "mem" => Ok(ProcColumn::Mem),
            "mem%" => Ok(ProcColumn::Mem),
            "pid" => Ok(ProcColumn::Pid),
            "count" => Ok(ProcColumn::Count),
            "name" => Ok(ProcColumn::Name),
            "command" => Ok(ProcColumn::Command),
            "read" | "r/s" | "rps" => Ok(ProcColumn::Read),
            "write" | "w/s" | "wps" => Ok(ProcColumn::Write),
            "tread" | "t.read" => Ok(ProcColumn::TotalRead),
            "twrite" | "t.write" => Ok(ProcColumn::TotalWrite),
            "state" => Ok(ProcColumn::State),
            "user" => Ok(ProcColumn::User),
            "time" => Ok(ProcColumn::Time),
            #[cfg(feature = "gpu")]
            "gmem" | "gmem%" => Ok(ProcColumn::GpuMem),
            #[cfg(feature = "gpu")]
            "gpu%" => Ok(ProcColumn::GpuPercent),
            _ => Err(serde::de::Error::custom("doesn't match any column type")),
        }
    }
}

impl From<&ProcColumn> for ProcWidgetColumn {
    fn from(value: &ProcColumn) -> Self {
        match value {
            ProcColumn::Pid => ProcWidgetColumn::PidOrCount,
            ProcColumn::Count => ProcWidgetColumn::PidOrCount,
            ProcColumn::Name => ProcWidgetColumn::ProcNameOrCommand,
            ProcColumn::Command => ProcWidgetColumn::ProcNameOrCommand,
            ProcColumn::CpuPercent => ProcWidgetColumn::Cpu,
            ProcColumn::Mem => ProcWidgetColumn::Mem,
            ProcColumn::MemPercent => ProcWidgetColumn::Mem,
            ProcColumn::Read => ProcWidgetColumn::ReadPerSecond,
            ProcColumn::Write => ProcWidgetColumn::WritePerSecond,
            ProcColumn::TotalRead => ProcWidgetColumn::TotalRead,
            ProcColumn::TotalWrite => ProcWidgetColumn::TotalWrite,
            ProcColumn::State => ProcWidgetColumn::State,
            ProcColumn::User => ProcWidgetColumn::User,
            ProcColumn::Time => ProcWidgetColumn::Time,
            ProcColumn::GpuMem => ProcWidgetColumn::GpuMem,
            ProcColumn::GpuPercent => ProcWidgetColumn::GpuUtil,
        }
    }
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
    fn process_column_settings() {
        let config = r#"
            columns = ["CPU%", "PiD", "user", "MEM", "Tread", "T.Write", "Rps", "W/s", "tiMe", "USER", "state"]
        "#;

        let generated: ProcessesConfig = toml_edit::de::from_str(config).unwrap();
        assert_eq!(
            to_columns(generated.columns),
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
