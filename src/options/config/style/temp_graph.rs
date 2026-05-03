use serde::{Deserialize, Serialize};

use super::ColorStr;

/// Styling specific to the temperature graph widget.
#[derive(Clone, Debug, Default, Deserialize, Serialize)]
#[cfg_attr(feature = "generate_schema", derive(schemars::JsonSchema))]
#[cfg_attr(test, serde(deny_unknown_fields), derive(PartialEq, Eq))]
pub(crate) struct TempGraphStyle {
    /// Colour of each temperature sensor's graph line. Read in order.
    #[serde(alias = "temp_graph_colour_styles")]
    pub(crate) temp_graph_color_styles: Option<Vec<ColorStr>>,
}
