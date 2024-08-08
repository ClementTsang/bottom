use serde::{Deserialize, Serialize};

use super::ColorStr;

/// Styling specific to the CPU widget.
#[derive(Clone, Debug, Default, Deserialize, Serialize)]
#[cfg_attr(feature = "generate_schema", derive(schemars::JsonSchema))]
#[cfg_attr(test, serde(deny_unknown_fields), derive(PartialEq, Eq))]
pub(crate) struct CpuStyle {
    /// The colour of the "All" CPU label.
    #[serde(alias = "all_entry_colour")]
    pub(crate) all_entry_color: Option<ColorStr>,

    /// The colour of the average CPU label and graph line.
    #[serde(alias = "avg_entry_colour")]
    pub(crate) avg_entry_color: Option<ColorStr>,

    /// Colour of each CPU threads' label and graph line. Read in order.
    #[serde(alias = "cpu_core_colours")]
    pub(crate) cpu_core_colors: Option<Vec<ColorStr>>,
}
