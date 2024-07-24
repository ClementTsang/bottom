use serde::{Deserialize, Serialize};

use super::ColorStr;

/// Styling specific to the CPU widget.
#[derive(Clone, Debug, Default, Deserialize, Serialize)]
#[cfg_attr(feature = "generate_schema", derive(schemars::JsonSchema))]
pub(crate) struct CpuStyle {
    #[serde(alias = "all_entry_colour")]
    pub(crate) all_entry_color: Option<ColorStr>,

    #[serde(alias = "avg_entry_colour")]
    pub(crate) avg_entry_color: Option<ColorStr>,

    #[serde(alias = "cpu_core_colours")]
    pub(crate) cpu_core_colors: Option<Vec<ColorStr>>,
}
