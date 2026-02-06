use serde::{Deserialize, Serialize};

use super::{ColorStr, TextStyleConfig, borders::WidgetBorderType};

/// General styling for generic widgets.
#[derive(Clone, Debug, Default, Deserialize, Serialize)]
#[cfg_attr(feature = "generate_schema", derive(schemars::JsonSchema))]
#[cfg_attr(test, serde(deny_unknown_fields), derive(PartialEq, Eq))]
pub(crate) struct WidgetStyle {
    /// The colour of the widgets' borders.
    #[serde(alias = "border_colour")]
    pub(crate) border_color: Option<ColorStr>,

    /// The colour of a widget's borders when the widget is selected.
    #[serde(alias = "selected_border_colour")]
    pub(crate) selected_border_color: Option<ColorStr>,

    /// Text styling for a widget's title.
    pub(crate) widget_title: Option<TextStyleConfig>,

    /// Text styling for text in general.
    pub(crate) text: Option<TextStyleConfig>,

    /// Text styling for text when representing something that is selected.
    pub(crate) selected_text: Option<TextStyleConfig>,

    /// Text styling for text when representing something that is disabled.
    pub(crate) disabled_text: Option<TextStyleConfig>,

    /// Text styling for text when representing process threads. Only usable
    /// on Linux at the moment.
    pub(crate) thread_text: Option<TextStyleConfig>,

    /// Widget borders type.
    pub(crate) widget_border_type: Option<WidgetBorderType>,

    /// Progress bar characters to use
    pub(crate) progress_bar_chars: Option<ProgressBarChars>,
}

#[derive(Clone, Debug)]
#[cfg_attr(test, derive(PartialEq, Eq))]
pub(crate) struct ProgressBarChars(pub(crate) Vec<char>);

impl<'de> Deserialize<'de> for ProgressBarChars {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        Vec::<char>::deserialize(deserializer).and_then(|chars| {
            if chars.is_empty() {
                Err(<D::Error as serde::de::Error>::custom(
                    "the array of progress bar characters must be non-empty",
                ))
            } else {
                Ok(Self(chars))
            }
        })
    }
}

impl Serialize for ProgressBarChars {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        self.0.serialize(serializer)
    }
}

#[cfg(feature = "generate_schema")]
impl schemars::JsonSchema for ProgressBarChars {
    fn schema_name() -> std::borrow::Cow<'static, str> {
        Vec::<char>::schema_name()
    }

    fn json_schema(generator: &mut schemars::SchemaGenerator) -> schemars::Schema {
        let mut schema = generator.subschema_for::<Vec<char>>();
        schema.insert(
            "minItems".into(),
            serde_json::Value::Number(serde_json::Number::from(1u64)),
        );
        schema
    }
}

impl Default for ProgressBarChars {
    fn default() -> Self {
        Self(vec!['▏', '▎', '▍', '▌', '▋', '▊', '▉', '█'])
    }
}
