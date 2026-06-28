use serde::{Deserialize, Serialize};

use super::{ColourStr, TextStyleConfig, borders::WidgetBorderType};

/// General styling for generic widgets.
#[derive(Clone, Debug, Default, Deserialize, Serialize)]
#[cfg_attr(feature = "generate_schema", derive(schemars::JsonSchema))]
#[cfg_attr(test, serde(deny_unknown_fields), derive(PartialEq, Eq))]
pub(crate) struct WidgetStyle {
    /// The colour of the widgets' borders.
    #[serde(alias = "border_color")]
    pub(crate) border_colour: Option<ColourStr>,

    /// The colour of a widget's borders when the widget is selected.
    #[serde(alias = "selected_border_color")]
    pub(crate) selected_border_colour: Option<ColourStr>,

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

    /// Background colour for widgets.
    #[serde(alias = "bg_color")]
    pub(crate) bg_colour: Option<ColourStr>,
}
