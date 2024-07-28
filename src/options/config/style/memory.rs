use serde::{Deserialize, Serialize};

use super::ColorStr;

/// Styling specific to the memory widget.
#[derive(Clone, Debug, Default, Deserialize, Serialize)]
#[cfg_attr(feature = "generate_schema", derive(schemars::JsonSchema))]
pub(crate) struct MemoryStyle {
    pub(crate) ram: Option<ColorStr>,
    #[cfg(not(target_os = "windows"))]
    pub(crate) cache: Option<ColorStr>,
    pub(crate) swap: Option<ColorStr>,
    pub(crate) arc: Option<ColorStr>,
    pub(crate) gpus: Option<Vec<ColorStr>>,
}
