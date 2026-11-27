//! Config options around styling.

mod bars;
mod battery;
mod borders;
mod cpu;
mod disk_io_graph;
mod graphs;
mod memory;
mod network;
mod tables;
mod temp_graph;
mod themes;
mod utils;
mod widgets;

use std::borrow::Cow;

use battery::BatteryStyle;
use cpu::CpuStyle;
use disk_io_graph::DiskIoGraphStyle;
use graphs::GraphStyle;
use memory::MemoryStyle;
use network::NetworkStyle;
use ratatui::{style::Style, widgets::BorderType};
use serde::{Deserialize, Serialize};
use tables::TableStyle;
use temp_graph::TempGraphStyle;
use terminal_colorsaurus::ThemeMode;
use utils::{opt, set_colour, set_colour_list, set_style};
use widgets::WidgetStyle;

use super::Config;
use crate::{
    canvas::components::pipe_gauge::BarType,
    options::{OptionError, OptionResult, args::BottomArgs, config::style::utils::set_bg_colour},
};

#[derive(Clone, Debug, Deserialize, Serialize)]
#[cfg_attr(feature = "generate_schema", derive(schemars::JsonSchema))]
#[cfg_attr(test, derive(PartialEq, Eq))]
pub(crate) struct ColourStr(Cow<'static, str>);

/// A style for text.
#[derive(Clone, Debug, Deserialize, Serialize)]
#[serde(untagged)]
#[cfg_attr(feature = "generate_schema", derive(schemars::JsonSchema))]
#[cfg_attr(test, serde(deny_unknown_fields), derive(PartialEq, Eq))]
pub(crate) enum TextStyleConfig {
    Colour(ColourStr),
    TextStyle {
        /// A built-in ANSI colour, RGB hex, or RGB colour code.
        #[serde(alias = "color")]
        colour: Option<ColourStr>,

        /// A built-in ANSI colour, RGB hex, or RGB colour code.
        #[serde(alias = "bg_color")]
        bg_colour: Option<ColourStr>,

        /// Whether to make this text bolded or not. If not set,
        /// will default to built-in defaults.
        bold: Option<bool>,

        /// Whether to make this text italicized or not. If not set,
        /// will default to built-in defaults.
        italics: Option<bool>,
    },
}

/// Style-related configs.
#[derive(Clone, Debug, Default, Deserialize, Serialize)]
#[cfg_attr(feature = "generate_schema", derive(schemars::JsonSchema))]
#[cfg_attr(test, serde(deny_unknown_fields), derive(PartialEq, Eq))]
pub(crate) struct StyleConfig {
    /// A built-in theme. Currently supported themes are:
    /// - default
    /// - default-light: default but adjusted for lighter backgrounds
    /// - gruvbox: a bright theme with 'retro groove' colours
    /// - gruvbox-light: gruvbox but adjusted for lighter backgrounds
    /// - nord: an arctic, north-bluish colour palette
    /// - nord-light: nord but adjusted for lighter backgrounds
    ///
    /// An explicitly selected built-in theme is used as-is rather than being
    /// combined with component-level styles from the config file.
    ///
    /// In addition, you can use special values to control dark/light detection:
    /// - auto: (default) automatically pick a dark or light theme based on the
    ///   terminal, controlled by '--theme-dark' and '--theme-light'
    ///   respectively
    /// - dark: force the dark theme set by '--theme-dark'
    /// - light: force the light theme set by '--theme-light'"
    ///
    /// Component-level styles are applied when no built-in theme is selected,
    /// or when using the `custom` value for `theme_dark` or `theme_light`.
    pub(crate) theme: Option<Cow<'static, str>>,

    /// Specify a theme to use when the terminal has a dark background
    /// or 'theme' is set to 'dark'. Defaults to using the 'default'
    /// theme.
    ///
    /// Note that if the '--theme/--theme_dark' args are used while this
    /// option is set in the configuration file, the arguments will take
    /// precedence.
    pub(crate) theme_dark: Option<Cow<'static, str>>,

    /// Specify a theme to use when the terminal has a light background
    /// or 'theme' is set to 'light'. Defaults to using the 'default-light'
    /// theme.
    ///
    /// Note that if the '--theme/--theme_light' args are used while this
    /// option is set in the configuration file, the arguments will take
    /// precedence.
    pub(crate) theme_light: Option<Cow<'static, str>>,

    /// Styling for the CPU widget.
    pub(crate) cpu: Option<CpuStyle>,

    /// Styling for the memory widget.
    pub(crate) memory: Option<MemoryStyle>,

    /// Styling for the network widget.
    pub(crate) network: Option<NetworkStyle>,

    /// Styling for the disk I/O graph widget.
    pub(crate) disk_io_graph: Option<DiskIoGraphStyle>,

    /// Styling for the temperature graph widget.
    pub(crate) temp_graph: Option<TempGraphStyle>,

    /// Styling for the battery widget.
    pub(crate) battery: Option<BatteryStyle>,

    /// Styling for table widgets.
    pub(crate) tables: Option<TableStyle>,

    /// Styling for graph widgets.
    pub(crate) graphs: Option<GraphStyle>,

    /// Styling for general widgets.
    pub(crate) widgets: Option<WidgetStyle>,
}

enum StyleMode {
    Auto,
    Dark,
    Light,
}

impl StyleMode {
    fn is_dark(style_mode: Self) -> bool {
        match style_mode {
            StyleMode::Auto => {
                terminal_colorsaurus::theme_mode(terminal_colorsaurus::QueryOptions::default())
                    .map(|mode| mode == ThemeMode::Dark)
                    .unwrap_or(true)
            }
            StyleMode::Dark => true,
            StyleMode::Light => false,
        }
    }
}

/// The actual internal representation of the configured styles.
#[derive(Debug)]
pub struct Styles {
    pub(crate) ram_style: Style,
    #[cfg(not(target_os = "windows"))]
    pub(crate) cache_style: Style,
    pub(crate) swap_style: Style,
    #[cfg(target_os = "linux")]
    pub(crate) zram_style: Style,
    #[cfg(target_os = "linux")]
    pub(crate) zswap_style: Style,
    #[cfg(feature = "zfs")]
    pub(crate) arc_style: Style,
    #[cfg(feature = "gpu")]
    pub(crate) gpu_colours: Vec<Style>,
    pub(crate) rx_style: Style,
    pub(crate) tx_style: Style,
    pub(crate) total_rx_style: Style,
    pub(crate) total_tx_style: Style,
    pub(crate) all_cpu_colour: Style,
    pub(crate) avg_cpu_colour: Style,
    pub(crate) cpu_colour_styles: Vec<Style>,
    pub(crate) temp_graph_colour_styles: Vec<Style>,
    pub(crate) disk_io_read_colour_styles: Vec<Style>,
    pub(crate) disk_io_write_colour_styles: Vec<Style>,
    pub(crate) border_style: Style,
    pub(crate) highlighted_border_style: Style,
    pub(crate) text_style: Style,
    pub(crate) selected_text_style: Style,
    pub(crate) table_header_style: Style,
    pub(crate) widget_title_style: Style,
    pub(crate) general_widget_style: Style,
    pub(crate) graph_style: Style,
    pub(crate) graph_legend_style: Style,
    pub(crate) high_battery: Style,
    pub(crate) medium_battery: Style,
    pub(crate) low_battery: Style,
    pub(crate) invalid_query_style: Style,
    pub(crate) disabled_text_style: Style,
    #[cfg(target_os = "linux")]
    pub(crate) thread_text_style: Style,
    pub(crate) border_type: BorderType,
    pub(crate) bar_type: BarType,
}

impl Default for Styles {
    fn default() -> Self {
        Self::default_palette()
    }
}

impl Styles {
    /// Resolve the correct styling to use throughout the application.
    /// This must take into account light/dark modes if configured.
    ///
    /// In terms of priority:
    /// - If `--theme` is set via arguments, then we will use that to determine
    ///   which theme to apply.
    /// - We then check if `--theme_dark` or `--theme_light` are set, depending
    ///   on what `--theme` was set to. If it was not set, then this is skipped.
    /// - If not, then either keep trying with the argument value of `--theme`
    ///   or check if the config has it set. If neither is set, defaults to
    ///   `auto`.
    /// - Then try resolving the config's `theme_dark` or `theme_light` as
    ///   appropriate.
    /// - If they weren't set, they will default to `default` or `default-light`
    ///   respectively, and then the custom palette from the config will be
    ///   applied on top if available.
    ///
    /// Note this is possibly subject to change! I may make it so you can define
    /// custom light and dark palettes instead, for example.
    pub fn new(args: &BottomArgs, config: &Config) -> anyhow::Result<Self> {
        // First, check if the theme is set via args.
        let arg_is_dark = {
            let arg_style_mode = if let Some(theme) = &args.style.theme {
                if let Ok(style) = Self::from_built_in_theme(theme) {
                    return Ok(style);
                } else {
                    Some(match theme.to_lowercase().as_str() {
                        "dark" => StyleMode::Dark,
                        "light" => StyleMode::Light,
                        "auto" => StyleMode::Auto,
                        _ => {
                            return Err(OptionError::other(format!(
                                "'{theme}' is an invalid argument."
                            )))?;
                        }
                    })
                }
            } else {
                None
            };

            arg_style_mode.map(StyleMode::is_dark)
        };

        // If the theme_dark/theme_light args or config options are set, resolve
        // them here.
        if let Some(is_dark) = arg_is_dark {
            let (arg_theme, config_theme) = if is_dark {
                (
                    args.style.theme_dark.as_deref(),
                    config.styles.as_ref().and_then(|s| s.theme_dark.as_deref()),
                )
            } else {
                (
                    args.style.theme_light.as_deref(),
                    config
                        .styles
                        .as_ref()
                        .and_then(|s| s.theme_light.as_deref()),
                )
            };

            if let Some(theme) = arg_theme {
                return Self::resolve_selected_theme(theme, config, is_dark);
            } else if let Some(theme) = config_theme {
                return Self::resolve_selected_theme(theme, config, is_dark);
            }
        }

        // At this point, if we still haven't set a theme, that must me we did
        // not set any theme via args, or we couldn't resolve a theme via
        // theme_light/theme_dark. If it wasn't set via arg, let's next check if
        // it was set via config.
        //
        // Note that if the theme is not explicitly set, we'll assume it's
        // "auto".
        let config_is_dark = match arg_is_dark {
            Some(is_dark) => is_dark,
            None => {
                let config_style_mode =
                    if let Some(theme) = config.styles.as_ref().and_then(|s| s.theme.as_ref()) {
                        if let Ok(style) = Self::from_built_in_theme(theme) {
                            return Ok(style);
                        } else {
                            match theme.to_lowercase().as_str() {
                                "dark" => StyleMode::Dark,
                                "light" => StyleMode::Light,
                                "auto" => StyleMode::Auto,
                                _ => {
                                    return Err(OptionError::other(format!(
                                        "'{theme}' is an invalid argument."
                                    )))?;
                                }
                            }
                        }
                    } else {
                        StyleMode::Auto
                    };

                StyleMode::is_dark(config_style_mode)
            }
        };

        let (arg_theme, config_theme) = if config_is_dark {
            (
                args.style.theme_dark.as_deref(),
                config.styles.as_ref().and_then(|s| s.theme_dark.as_deref()),
            )
        } else {
            (
                args.style.theme_light.as_deref(),
                config
                    .styles
                    .as_ref()
                    .and_then(|s| s.theme_light.as_deref()),
            )
        };

        if let Some(theme) = arg_theme {
            Self::resolve_selected_theme(theme, config, config_is_dark)
        } else if let Some(theme) = config_theme {
            Self::resolve_selected_theme(theme, config, config_is_dark)
        } else {
            // If we somehow did not resolve the theme still, then automatically
            // the base via the default theme and finish resolving with any
            // custom colour palette.
            Self::resolve_theme_from_config(config, config_is_dark)
        }
    }

    fn resolve_theme_from_config(config: &Config, is_dark: bool) -> anyhow::Result<Self> {
        let palette = if is_dark {
            Self::default_palette()
        } else {
            Self::default_light_palette()
        };

        Self::apply_config_styles(palette, config)
    }

    fn resolve_selected_theme(theme: &str, config: &Config, is_dark: bool) -> anyhow::Result<Self> {
        match theme {
            "custom" => Self::resolve_theme_from_config(config, is_dark),
            _ => Self::from_built_in_theme(theme),
        }
    }

    fn apply_config_styles(mut palette: Self, config: &Config) -> anyhow::Result<Self> {
        if let Some(config_style) = &config.styles {
            palette.set_styles_from_config(config_style)?;
        }

        Ok(palette)
    }

    fn from_built_in_theme(theme: &str) -> anyhow::Result<Self> {
        let lower_case = theme.to_lowercase();
        match lower_case.as_str() {
            "default" => Ok(Self::default_palette()),
            "default-light" => Ok(Self::default_light_palette()),
            "gruvbox" => Ok(Self::gruvbox_palette()),
            "gruvbox-light" => Ok(Self::gruvbox_light_palette()),
            "nord" => Ok(Self::nord_palette()),
            "nord-light" => Ok(Self::nord_light_palette()),
            _ => Err(OptionError::other(format!(
                "'{theme}' is an invalid built-in colour scheme."
            ))
            .into()),
        }
    }

    fn set_styles_from_config(&mut self, config: &StyleConfig) -> OptionResult<()> {
        // CPU
        set_colour!(self.avg_cpu_colour, config.cpu, avg_entry_colour);
        set_colour!(self.all_cpu_colour, config.cpu, all_entry_colour);
        set_colour_list!(self.cpu_colour_styles, config.cpu, cpu_core_colours);

        // Temperature graph
        set_colour_list!(
            self.temp_graph_colour_styles,
            config.temp_graph,
            temp_graph_colour_styles
        );

        // Disk I/O graph
        set_colour_list!(
            self.disk_io_read_colour_styles,
            config.disk_io_graph,
            read_colours
        );
        set_colour_list!(
            self.disk_io_write_colour_styles,
            config.disk_io_graph,
            write_colours
        );

        // Memory
        set_colour!(self.ram_style, config.memory, ram_colour);
        set_colour!(self.swap_style, config.memory, swap_colour);

        #[cfg(not(target_os = "windows"))]
        set_colour!(self.cache_style, config.memory, cache_colour);

        #[cfg(target_os = "linux")]
        set_colour!(self.zram_style, config.memory, zram_colour);

        #[cfg(target_os = "linux")]
        set_colour!(self.zswap_style, config.memory, zswap_colour);

        #[cfg(feature = "zfs")]
        set_colour!(self.arc_style, config.memory, arc_colour);

        #[cfg(feature = "gpu")]
        set_colour_list!(self.gpu_colours, config.memory, gpu_colours);

        // Network
        set_colour!(self.rx_style, config.network, rx_colour);
        set_colour!(self.tx_style, config.network, tx_colour);
        set_colour!(self.total_rx_style, config.network, rx_total_colour);
        set_colour!(self.total_tx_style, config.network, tx_total_colour);

        // Battery
        set_colour!(self.high_battery, config.battery, high_battery_colour);
        set_colour!(self.medium_battery, config.battery, medium_battery_colour);
        set_colour!(self.low_battery, config.battery, low_battery_colour);

        // Tables
        set_style!(self.table_header_style, config.tables, headers);

        // Widget graphs
        set_colour!(self.graph_style, config.graphs, graph_colour);
        set_style!(self.graph_legend_style, config.graphs, legend_text);

        // General widget text.
        set_bg_colour!(self.general_widget_style, config.widgets, bg_colour);
        set_style!(self.widget_title_style, config.widgets, widget_title);
        set_style!(self.text_style, config.widgets, text);
        set_style!(self.selected_text_style, config.widgets, selected_text);
        set_style!(self.disabled_text_style, config.widgets, disabled_text);

        #[cfg(target_os = "linux")]
        {
            set_style!(self.thread_text_style, config.widgets, thread_text);
        }

        // Widget borders
        set_colour!(self.border_style, config.widgets, border_colour);
        set_colour!(
            self.highlighted_border_style,
            config.widgets,
            selected_border_colour
        );

        if let Some(widgets) = &config.widgets {
            if let Some(widget_borders) = widgets.widget_border_type {
                self.border_type = widget_borders.into();
            }

            if let Some(bar_type) = widgets.bar_type {
                self.bar_type = bar_type.into();
            }
        }

        Ok(())
    }
}

#[cfg(test)]
mod test {

    use clap::Parser;
    use ratatui::style::{Color, Style};

    use super::*;
    use crate::options::config::style::utils::str_to_colour;

    #[test]
    fn color_spelling_aliases_work() {
        // Parse all_styling_color.toml (which uses the "color" alias spelling
        // throughout) and assert every field was actually parsed.
        let toml_str = include_str!("../../../tests/valid_configs/all_styling_color.toml");
        let config = toml_edit::de::from_str::<Config>(toml_str)
            .expect("config file should parse")
            .styles
            .expect("styles section should be present");

        let cpu = config.cpu.as_ref().unwrap();
        assert!(cpu.all_entry_colour.is_some());
        assert!(cpu.avg_entry_colour.is_some());
        assert!(cpu.cpu_core_colours.is_some());

        let temp = config.temp_graph.as_ref().unwrap();
        assert!(temp.temp_graph_colour_styles.is_some());

        let mem = config.memory.as_ref().unwrap();
        assert!(mem.ram_colour.is_some());
        assert!(mem.cache_colour.is_some());
        assert!(mem.swap_colour.is_some());
        assert!(mem.zram_colour.is_some());
        assert!(mem.zswap_colour.is_some());
        assert!(mem.arc_colour.is_some());
        assert!(mem.gpu_colours.is_some());

        let net = config.network.as_ref().unwrap();
        assert!(net.rx_colour.is_some());
        assert!(net.tx_colour.is_some());
        assert!(net.rx_total_colour.is_some());
        assert!(net.tx_total_colour.is_some());

        let bat = config.battery.as_ref().unwrap();
        assert!(bat.high_battery_colour.is_some());
        assert!(bat.medium_battery_colour.is_some());
        assert!(bat.low_battery_colour.is_some());

        let graphs = config.graphs.as_ref().unwrap();
        assert!(graphs.graph_colour.is_some());

        let widgets = config.widgets.as_ref().unwrap();
        assert!(widgets.border_colour.is_some());
        assert!(widgets.selected_border_colour.is_some());
        assert!(widgets.bg_colour.is_some());
    }

    #[test]
    fn default_selected_colour_works() {
        let mut colours = Styles::default();
        let original_selected_text_colour =
            Styles::default_palette().selected_text_style.fg.unwrap();
        let original_selected_bg_colour = Styles::default_palette().selected_text_style.bg.unwrap();

        assert_eq!(
            colours.selected_text_style,
            Style::default()
                .fg(original_selected_text_colour)
                .bg(original_selected_bg_colour),
        );

        colours.selected_text_style = colours
            .selected_text_style
            .fg(str_to_colour("magenta").unwrap())
            .bg(str_to_colour("red").unwrap());

        assert_eq!(
            colours.selected_text_style,
            Style::default().fg(Color::Magenta).bg(Color::Red),
        );
    }

    #[test]
    fn auto_theme_defaults() {
        let config = toml_edit::de::from_str::<Config>(
            r#"[styles]
            theme = "auto"
            "#,
        )
        .expect("auto theme config should parse");

        let dark = Styles::resolve_theme_from_config(&config, true).unwrap();
        let light = Styles::resolve_theme_from_config(&config, false).unwrap();

        assert_eq!(dark.ram_style, Styles::default_palette().ram_style);
        assert_eq!(light.ram_style, Styles::default_light_palette().ram_style);
    }

    #[test]
    fn auto_theme_with_custom() {
        let args = BottomArgs::try_parse_from(["btm"]).expect("default arguments should parse");
        let config = toml_edit::de::from_str::<Config>(
            r#"[styles]
            theme = "auto"

            [styles.cpu]
            avg_entry_colour = "red"
            "#,
        )
        .expect("auto theme config should parse");

        let styles = Styles::new(&args, &config).expect("auto theme should resolve");

        assert_eq!(styles.avg_cpu_colour.fg, Some(Color::Red));
    }

    /// Ensures that built-in themes are _not_ merged with custom styles if
    /// defined using `theme`.
    #[test]
    fn built_in_theme_merging() {
        let args = BottomArgs::try_parse_from(["btm"]).expect("default arguments should parse");
        let config = toml_edit::de::from_str::<Config>(
            r#"[styles]
            theme = "nord"

            [styles.cpu]
            avg_entry_colour = "red"
            "#,
        )
        .expect("theme config should parse");

        let styles = Styles::new(&args, &config).expect("configured theme should resolve");

        assert_eq!(styles.avg_cpu_colour, Styles::nord_palette().avg_cpu_colour);
    }

    #[test]
    fn built_in_colour_schemes_work() {
        Styles::from_built_in_theme("default").unwrap();
        Styles::from_built_in_theme("default-light").unwrap();
        Styles::from_built_in_theme("gruvbox").unwrap();
        Styles::from_built_in_theme("gruvbox-light").unwrap();
        Styles::from_built_in_theme("nord").unwrap();
        Styles::from_built_in_theme("nord-light").unwrap();
    }
}
