use serde::{Deserialize, Serialize};

use crate::canvas::components::pipe_gauge::BarType;

/// The type of character used to fill in bars, such as the ones used by the
/// basic CPU and memory widgets.
#[derive(Default, Clone, Copy, Debug, Serialize)]
#[serde(rename_all = "lowercase")]
#[cfg_attr(feature = "generate_schema", derive(schemars::JsonSchema))]
#[cfg_attr(test, derive(PartialEq, Eq))]
pub(crate) enum WidgetBarType {
    /// Fill bars with pipe characters (`|`).
    #[default]
    Pipe,
    /// Fill bars with block characters (`█`, `▉`, etc.).
    Solid,
}

impl<'de> Deserialize<'de> for WidgetBarType {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        let value = String::deserialize(deserializer)?.to_lowercase();
        match value.as_str() {
            "pipe" => Ok(WidgetBarType::Pipe),
            "solid" => Ok(WidgetBarType::Solid),
            _ => Err(serde::de::Error::custom("doesn't match any bar type")),
        }
    }
}

impl From<WidgetBarType> for BarType {
    fn from(value: WidgetBarType) -> Self {
        match value {
            WidgetBarType::Pipe => BarType::Pipe,
            WidgetBarType::Solid => BarType::Bar,
        }
    }
}

#[cfg(test)]
mod test {
    use super::WidgetBarType;
    use crate::options::config::style::widgets::WidgetStyle;

    fn parse(value: &str) -> anyhow::Result<Option<WidgetBarType>> {
        let style = toml_edit::de::from_str::<WidgetStyle>(&format!("bar_type = {value}"))?;

        Ok(style.bar_type)
    }

    #[test]
    fn valid_bar_types() {
        assert_eq!(parse("\"pipe\"").unwrap(), Some(WidgetBarType::Pipe));
        assert_eq!(parse("\"solid\"").unwrap(), Some(WidgetBarType::Solid));

        // Casing shouldn't matter.
        assert_eq!(parse("\"Solid\"").unwrap(), Some(WidgetBarType::Solid));
        assert_eq!(parse("\"PIPE\"").unwrap(), Some(WidgetBarType::Pipe));
    }

    #[test]
    fn invalid_bar_types() {
        assert!(parse("\"bar\"").is_err());
        assert!(parse("\"\"").is_err());
        assert!(parse("true").is_err());
    }
}
