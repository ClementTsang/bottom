use serde::{Deserialize, Serialize};

use super::{ColorStr, TextStyleConfig};

/// Styling specific to the memory widget.
#[derive(Clone, Debug, Default, Deserialize, Serialize)]
#[cfg_attr(feature = "generate_schema", derive(schemars::JsonSchema))]
pub(crate) struct MemoryStyle {
    pub(crate) ram: Option<TextStyleConfig>, // TODO: Hm how do we want to do this, colours only, text...
    #[cfg(not(target_os = "windows"))]
    pub(crate) cache: Option<TextStyleConfig>,
    pub(crate) swap: Option<TextStyleConfig>,
    pub(crate) arc: Option<TextStyleConfig>,
    pub(crate) gpus: Option<Vec<ColorStr>>,
}
