use serde::{Deserialize, Serialize};

use super::TextStyleConfig;

/// General styling for table widgets.
#[derive(Clone, Debug, Default, Deserialize, Serialize)]
#[cfg_attr(feature = "generate_schema", derive(schemars::JsonSchema))]
pub(crate) struct TableStyle {
    /// Text styling for table headers.
    pub(crate) headers: Option<TextStyleConfig>,
}
