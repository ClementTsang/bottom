use serde::{Deserialize, Serialize};

use super::ColourStr;

/// Styling specific to the disk space graph widget.
#[derive(Clone, Debug, Default, Deserialize, Serialize)]
#[cfg_attr(feature = "generate_schema", derive(schemars::JsonSchema))]
#[cfg_attr(test, serde(deny_unknown_fields), derive(PartialEq, Eq))]
pub(crate) struct DiskSpaceGraphStyle {
    /// Colour of each disk's used-space graph line. Read in order.
    pub(crate) colours: Option<Vec<ColourStr>>,
}
