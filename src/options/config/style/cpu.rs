use serde::{Deserialize, Serialize};

use super::TextStyleConfig;

/// Styling specific to the CPU widget.
#[derive(Clone, Debug, Default, Deserialize, Serialize)]
#[cfg_attr(feature = "generate_schema", derive(schemars::JsonSchema))]
pub(crate) struct CpuStyle {
    pub(crate) all_cpu: Option<TextStyleConfig>,
    pub(crate) avg_cpu: Option<TextStyleConfig>,
    pub(crate) cpu_core: Option<TextStyleConfig>,
}
