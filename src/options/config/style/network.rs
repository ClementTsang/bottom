use serde::{Deserialize, Serialize};

use super::ColorStr;

/// Styling specific to the network widget.
#[derive(Clone, Debug, Default, Deserialize, Serialize)]
#[cfg_attr(feature = "generate_schema", derive(schemars::JsonSchema))]
#[cfg_attr(test, serde(deny_unknown_fields), derive(PartialEq, Eq))]
pub(crate) struct NetworkStyle {
    /// The colour of the RX (download) label and graph line.
    #[serde(alias = "rx_colour")]
    pub(crate) rx_color: Option<ColorStr>,

    /// The colour of the TX (upload) label and graph line.
    #[serde(alias = "tx_colour")]
    pub(crate) tx_color: Option<ColorStr>,

    /// he colour of the total RX (download) label in basic mode.
    #[serde(alias = "rx_total_colour")]
    pub(crate) rx_total_color: Option<ColorStr>,

    /// The colour of the total TX (upload) label in basic mode.
    #[serde(alias = "tx_total_colour")]
    pub(crate) tx_total_color: Option<ColorStr>,
}
