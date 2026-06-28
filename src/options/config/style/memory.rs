use serde::{Deserialize, Serialize};

use super::ColourStr;

/// Styling specific to the memory widget.
#[derive(Clone, Debug, Default, Deserialize, Serialize)]
#[cfg_attr(feature = "generate_schema", derive(schemars::JsonSchema))]
#[cfg_attr(test, serde(deny_unknown_fields), derive(PartialEq, Eq))]
pub(crate) struct MemoryStyle {
    /// The colour of the RAM label and graph line.
    #[serde(alias = "ram_color")]
    pub(crate) ram_colour: Option<ColourStr>,

    /// The colour of the cache label and graph line. Does not do anything on
    /// Windows.
    #[cfg_attr(target_os = "windows", allow(dead_code))]
    #[serde(alias = "cache_color")]
    pub(crate) cache_colour: Option<ColourStr>,

    /// The colour of the swap label and graph line.
    #[serde(alias = "swap_color")]
    pub(crate) swap_colour: Option<ColourStr>,

    /// The colour of the ARC label and graph line.
    #[serde(alias = "arc_color")]
    pub(crate) arc_colour: Option<ColourStr>,

    /// Colour of each GPU's memory label and graph line. Read in order.
    #[serde(alias = "gpu_colors")]
    pub(crate) gpu_colours: Option<Vec<ColourStr>>,
}
