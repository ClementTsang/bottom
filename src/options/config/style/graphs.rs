use serde::{Deserialize, Serialize};

use super::{ColorStr, TextStyleConfig};

/// General styling for graph widgets.
#[derive(Clone, Debug, Default, Deserialize, Serialize)]
#[cfg_attr(feature = "generate_schema", derive(schemars::JsonSchema))]
#[cfg_attr(test, serde(deny_unknown_fields), derive(PartialEq, Eq))]
pub(crate) struct GraphStyle {
    /// The general colour of the parts of the graph.
    #[serde(alias = "graph_colour")]
    pub(crate) graph_color: Option<ColorStr>,

    /// Text styling for graph's legend text.
    pub(crate) legend_text: Option<TextStyleConfig>,
}
