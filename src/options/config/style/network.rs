use serde::{Deserialize, Serialize};

use super::TextStyleConfig;

/// Styling specific to the network widget.
#[derive(Clone, Debug, Default, Deserialize, Serialize)]
#[cfg_attr(feature = "generate_schema", derive(schemars::JsonSchema))]
pub(crate) struct NetworkStyle {
    pub(crate) rx: Option<TextStyleConfig>,
    pub(crate) tx: Option<TextStyleConfig>,

    /// Set the colour of the "rx total" text. This only affects
    /// basic mode.
    pub(crate) rx_total: Option<TextStyleConfig>,

    /// Set the colour of the "tx total" text. This only affects
    /// basic mode.
    pub(crate) tx_total: Option<TextStyleConfig>,
}
