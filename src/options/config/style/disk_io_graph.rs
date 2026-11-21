use serde::{Deserialize, Serialize};

use super::ColourStr;

/// Styling specific to the disk I/O graph widget.
#[derive(Clone, Debug, Default, Deserialize, Serialize)]
#[cfg_attr(feature = "generate_schema", derive(schemars::JsonSchema))]
#[cfg_attr(test, serde(deny_unknown_fields), derive(PartialEq, Eq))]
pub(crate) struct DiskIoGraphStyle {
    /// Colour of each disk's read rate graph line. Read in order.
    #[serde(alias = "read_colours")]
    pub(crate) read_colours: Option<Vec<ColourStr>>,

    /// Colour of each disk's write rate graph line. Read in order.
    #[serde(alias = "write_colours")]
    pub(crate) write_colours: Option<Vec<ColourStr>>,
}
