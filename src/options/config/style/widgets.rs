use serde::{Deserialize, Serialize};

use super::{ColorStr, TextStyleConfig};

/// General styling for generic widgets.
#[derive(Clone, Debug, Default, Deserialize, Serialize)]
#[cfg_attr(feature = "generate_schema", derive(schemars::JsonSchema))]
pub(crate) struct WidgetStyle {
    pub(crate) border: Option<ColorStr>,
    pub(crate) selected_border: Option<ColorStr>,
    pub(crate) widget_title: Option<TextStyleConfig>,

    pub(crate) text: Option<TextStyleConfig>,
    pub(crate) selected_text: Option<TextStyleConfig>,
    pub(crate) disabled_text: Option<TextStyleConfig>,
}
