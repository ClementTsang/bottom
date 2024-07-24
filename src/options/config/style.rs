//! Config options around styling.

mod battery;
mod cpu;
mod graph;
mod memory;
mod network;
mod table;
mod themes;
mod widget;

use std::borrow::Cow;

use battery::BatteryStyle;
use cpu::CpuStyle;
use graph::GraphStyle;
use memory::MemoryStyle;
use network::NetworkStyle;
use serde::{Deserialize, Serialize};
use table::TableStyle;
use widget::WidgetStyle;

#[derive(Clone, Debug, Deserialize, Serialize)]
pub(crate) struct Color(Cow<'static, str>);

/// A style for text.
#[derive(Clone, Debug, Default, Deserialize, Serialize)]
#[cfg_attr(feature = "generate_schema", derive(schemars::JsonSchema))]
pub(crate) struct TextStyleConfig {
    /// A built-in ANSI colour, RGB hex, or RGB colour code.
    #[serde(alias = "colour")]
    pub(crate) color: Option<Color>,

    /// A built-in ANSI colour, RGB hex, or RGB colour code.
    #[serde(alias = "bg_colour")]
    pub(crate) bg_color: Option<Color>,

    /// Whether to make this text bolded or not. If not set,
    /// will default to built-in defaults.
    pub(crate) bold: Option<bool>,
}

/// Style-related configs.
#[derive(Clone, Debug, Default, Deserialize, Serialize)]
#[cfg_attr(feature = "generate_schema", derive(schemars::JsonSchema))]
pub(crate) struct StyleConfig {
    /// A built-in theme.
    ///
    /// If this is and a custom colour are both set, in the config file,
    /// the custom colour scheme will be prioritized first. If a theme
    /// is set in the command-line args, however, it will always be
    /// prioritized first.
    pub(crate) theme: Option<Cow<'static, str>>,

    /// Styling for the CPU widget.
    pub(crate) cpu: Option<CpuStyle>,

    /// Styling for the memory widget.
    pub(crate) memory: Option<MemoryStyle>,

    /// Styling for the network widget.
    pub(crate) network: Option<NetworkStyle>,

    /// Styling for the battery widget.
    pub(crate) battery: Option<BatteryStyle>,

    /// Styling for table widgets.
    pub(crate) tables: Option<TableStyle>,

    /// Styling for graph widgets.
    pub(crate) graphs: Option<GraphStyle>,

    /// Styling for general widgets.
    pub(crate) widgets: Option<WidgetStyle>,
}

impl StyleConfig {
    /// Returns `true` if there is a [`ConfigColours`] that is empty or there
    /// isn't one at all.
    pub(crate) fn is_empty(&self) -> bool {
        if let Ok(serialized_string) = toml_edit::ser::to_string(self) {
            return serialized_string.is_empty();
        }

        true
    }
}
