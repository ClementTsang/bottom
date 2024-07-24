//! Config options around styling.

mod battery;
mod cpu;
mod graph;
mod memory;
mod network;
mod table;
mod themes;
mod utils;
mod widget;

use std::borrow::Cow;

use battery::BatteryStyle;
use cpu::CpuStyle;
use graph::GraphStyle;
use memory::MemoryStyle;
use network::NetworkStyle;
use serde::{Deserialize, Serialize};
use table::TableStyle;
use tui::style::Style;
use utils::{opt, try_set_colour, try_set_colour_list, try_set_style};
use widget::WidgetStyle;

use crate::options::{args::BottomArgs, OptionError, OptionResult};

use super::Config;

#[derive(Clone, Debug, Deserialize, Serialize)]
pub(crate) struct ColorStr(Cow<'static, str>);

/// A style for text.
#[derive(Clone, Debug, Default, Deserialize, Serialize)]
#[cfg_attr(feature = "generate_schema", derive(schemars::JsonSchema))]
pub(crate) struct TextStyleConfig {
    /// A built-in ANSI colour, RGB hex, or RGB colour code.
    #[serde(alias = "colour")]
    pub(crate) color: Option<ColorStr>,

    /// A built-in ANSI colour, RGB hex, or RGB colour code.
    #[serde(alias = "bg_colour")]
    pub(crate) bg_color: Option<ColorStr>,

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

/// The actual internal representation of the configured colours,
/// as a "palette".
pub(crate) struct ColourPalette {
    pub selected_text_style: Style,
    pub table_header_style: Style,
    pub ram_style: Style,
    #[cfg(not(target_os = "windows"))]
    pub cache_style: Style,
    pub swap_style: Style,
    pub arc_style: Style,
    pub gpu_colours: Vec<Style>,
    pub rx_style: Style,
    pub tx_style: Style,
    pub total_rx_style: Style,
    pub total_tx_style: Style,
    pub all_cpu_colour: Style,
    pub avg_cpu_colour: Style,
    pub cpu_colour_styles: Vec<Style>,
    pub border_style: Style,
    pub highlighted_border_style: Style,
    pub text_style: Style,
    pub widget_title_style: Style,
    pub graph_style: Style,
    pub high_battery_colour: Style,
    pub medium_battery_colour: Style,
    pub low_battery_colour: Style,
    pub invalid_query_style: Style,
    pub disabled_text_style: Style,
}

impl Default for ColourPalette {
    fn default() -> Self {
        Self::default_palette()
    }
}

impl ColourPalette {
    pub fn new(args: &BottomArgs, config: &Config) -> anyhow::Result<Self> {
        let mut palette = match &args.style.color {
            Some(theme) => Self::from_theme(theme)?,
            None => match config.style.as_ref().and_then(|s| s.theme.as_ref()) {
                Some(theme) => Self::from_theme(theme)?,
                None => Self::default(),
            },
        };

        // Apply theme from config on top.
        if let Some(style) = &config.style {
            palette.set_colours_from_palette(style);
        }

        Ok(palette)
    }

    fn from_theme(theme: &str) -> anyhow::Result<Self> {
        let lower_case = theme.to_lowercase();
        match lower_case.as_str() {
            "default" => Ok(Self::default_palette()),
            "default-light" => Ok(Self::default_light_mode()),
            "gruvbox" => Ok(todo!()),
            "gruvbox-light" => Ok(todo!()),
            "nord" => Ok(todo!()),
            "nord-light" => Ok(todo!()),
            _ => Err(
                OptionError::other(format!("'{theme}' is an invalid built-in color scheme."))
                    .into(),
            ),
        }
    }

    fn set_colours_from_palette(&mut self, config: &StyleConfig) -> OptionResult<()> {
        // CPU
        try_set_colour!(self.avg_cpu_colour, opt!(config.cpu?.avg_entry_color));
        try_set_colour!(self.all_cpu_colour, opt!(config.cpu?.all_entry_color));
        try_set_colour_list!(self.cpu_colour_styles, opt!(config.cpu?.cpu_core_colors));

        // Memory
        try_set_style!(self.ram_style, opt!(config.memory?.ram));
        try_set_style!(self.swap_style, opt!(config.memory?.swap));

        #[cfg(not(target_os = "windows"))]
        try_set_style!(self.cache_style, opt!(config.memory?.cache));

        #[cfg(feature = "zfs")]
        try_set_style!(self.arc_style, opt!(config.memory?.arc));

        #[cfg(feature = "gpu")]
        try_set_colour_list!(self.gpu_colours, opt!(config.memory?.gpus));

        // Network
        try_set_style!(self.rx_style, config, rx_color);
        try_set_style!(self.tx_style, config, tx_color);
        try_set_style!(self.total_rx_style, config, rx_total_color);
        try_set_style!(self.total_tx_style, config, tx_total_color);

        // Battery
        try_set_style!(self.high_battery_colour, config, high_battery_color);
        try_set_style!(self.medium_battery_colour, config, medium_battery_color);
        try_set_style!(self.low_battery_colour, config, low_battery_color);

        // Widget text and graphs
        try_set_style!(self.widget_title_style, config, widget_title_color);
        try_set_style!(self.graph_style, config, graph_color);
        try_set_style!(self.text_style, config, text_color);
        try_set_style!(self.disabled_text_style, config, disabled_text_color);
        try_set_style!(self.border_style, config, border_color);
        try_set_style!(
            self.highlighted_border_style,
            config,
            highlighted_border_color
        );

        // Tables
        try_set_style!(self.table_header_style, config, table_header_color);

        if let Some(scroll_entry_text_color) = &config.selected_text_color {
            self.set_selected_text_fg(scroll_entry_text_color)
                .map_err(|err| {
                    OptionError::config(format!(
                        "Please update 'colors.selected_text_color' in your config file. {err}",
                    ))
                })?
        }

        if let Some(scroll_entry_bg_color) = &config.selected_bg_color {
            self.set_selected_text_bg(scroll_entry_bg_color)
                .map_err(|err| {
                    OptionError::config(format!(
                        "Please update 'colors.selected_bg_color' in your config file. {err}",
                    ))
                })?
        }

        Ok(())
    }

    /// Set the selected text style's foreground colour.
    #[inline]
    fn set_selected_text_fg(&mut self, colour: &str) -> Result<(), String> {
        self.selected_text_style = self.selected_text_style.fg(str_to_colour(colour)?);
        Ok(())
    }

    /// Set the selected text style's background colour.
    #[inline]
    fn set_selected_text_bg(&mut self, colour: &str) -> Result<(), String> {
        self.selected_text_style = self.selected_text_style.bg(str_to_colour(colour)?);
        Ok(())
    }
}
