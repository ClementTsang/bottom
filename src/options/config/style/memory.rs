use serde::{Deserialize, Serialize};

use super::ColorStr;
// TODO: Maybe I should swap the alias and the field name since internally I use u.

/// Styling specific to the memory widget.
#[derive(Clone, Debug, Default, Deserialize, Serialize)]
#[cfg_attr(feature = "generate_schema", derive(schemars::JsonSchema))]
#[cfg_attr(test, serde(deny_unknown_fields), derive(PartialEq, Eq))]
pub(crate) struct MemoryStyle {
    /// The colour of the RAM label and graph line.
    #[serde(alias = "ram_colour")]
    pub(crate) ram_color: Option<ColorStr>,

    /// The colour of the cache label and graph line. Does not do anything on Windows.
    #[cfg_attr(target_os = "windows", allow(dead_code))]
    #[serde(alias = "cache_colour")]
    pub(crate) cache_color: Option<ColorStr>,

    /// The colour of the swap label and graph line.
    #[serde(alias = "swap_colour")]
    pub(crate) swap_color: Option<ColorStr>,

    /// The colour of the ARC label and graph line.
    #[serde(alias = "arc_colour")]
    pub(crate) arc_color: Option<ColorStr>,

    /// Colour of each GPU's memory label and graph line. Read in order.
    #[serde(alias = "gpu_colours")]
    pub(crate) gpu_colors: Option<Vec<ColorStr>>,
}
