use serde::{Deserialize, Serialize};

use super::{ColorStr, TextStyleConfig};

/// General styling for graph widgets.
#[derive(Clone, Debug, Default, Deserialize, Serialize)]
#[cfg_attr(feature = "generate_schema", derive(schemars::JsonSchema))]
pub(crate) struct GraphStyle {
    #[serde(alias = "graph_colour")]
    pub(crate) graph_color: Option<ColorStr>,

    pub(crate) legend_text: Option<TextStyleConfig>,
}
