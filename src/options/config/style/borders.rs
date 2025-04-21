use serde::{Deserialize, Serialize};
use tui::widgets::BorderType;

#[derive(Default, Clone, Copy, Debug, Serialize)]
#[cfg_attr(feature = "generate_schema", derive(schemars::JsonSchema))]
#[cfg_attr(test, derive(PartialEq, Eq))]
pub(crate) enum WidgetBorderType {
    #[default]
    Default,
    Rounded,
    Double,
    Thick,
}

impl<'de> Deserialize<'de> for WidgetBorderType {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        let value = String::deserialize(deserializer)?.to_lowercase();
        match value.as_str() {
            "default" => Ok(WidgetBorderType::Default),
            "rounded" => Ok(WidgetBorderType::Rounded),
            "double" => Ok(WidgetBorderType::Double),
            "thick" => Ok(WidgetBorderType::Thick),
            _ => Err(serde::de::Error::custom(
                "doesn't match any widget border type",
            )),
        }
    }
}

impl From<WidgetBorderType> for BorderType {
    fn from(value: WidgetBorderType) -> Self {
        match value {
            WidgetBorderType::Default => BorderType::Plain,
            WidgetBorderType::Rounded => BorderType::Rounded,
            WidgetBorderType::Double => BorderType::Double,
            WidgetBorderType::Thick => BorderType::Thick,
        }
    }
}
