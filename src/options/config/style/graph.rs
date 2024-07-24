use serde::{Deserialize, Serialize};

use super::Color;

/// General styling for graph widgets.
#[derive(Clone, Debug, Default, Deserialize, Serialize)]
#[cfg_attr(feature = "generate_schema", derive(schemars::JsonSchema))]
pub(crate) struct GraphStyle {
    pub(crate) graph: Option<Color>,
}
