use serde::{Deserialize, Serialize};

use super::TextStyleConfig;

/// General styling for table widgets.
#[derive(Clone, Debug, Default, Deserialize, Serialize)]
#[cfg_attr(feature = "generate_schema", derive(schemars::JsonSchema))]
#[cfg_attr(test, serde(deny_unknown_fields), derive(PartialEq, Eq))]
pub(crate) struct TableStyle {
    /// Text styling for table headers.
    pub(crate) headers: Option<TextStyleConfig>,
}
