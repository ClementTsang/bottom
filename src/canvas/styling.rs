mod colour_utils;

use colour_utils::*;
use tui::style::{Color, Style};

use super::ColourScheme;
pub use crate::options::ConfigV1;
use crate::{
    constants::*,
    options::{colours::ColoursConfig, OptionError, OptionResult},
};

pub struct CanvasStyles {
    pub selected_text_style: Style,
    pub table_header_style: Style,
    pub ram_style: Style,
    #[cfg(not(target_os = "windows"))]
    pub cache_style: Style,
    pub swap_style: Style,
    pub arc_style: Style,
    pub gpu_colour_styles: Vec<Style>,
    pub rx_style: Style,
    pub tx_style: Style,
    pub total_rx_style: Style,
    pub total_tx_style: Style,
    pub all_colour_style: Style,
    pub avg_colour_style: Style,
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

impl CanvasStyles {
    const DEFAULT_SELECTED_TEXT_STYLE: Style = Style::new().fg(Color::Black).bg(HIGHLIGHT_COLOUR);
}

impl Default for CanvasStyles {
    fn default() -> Self {
        let text_colour = Color::Gray;

        CanvasStyles {
            selected_text_style: CanvasStyles::DEFAULT_SELECTED_TEXT_STYLE,
            table_header_style: Style::default().fg(HIGHLIGHT_COLOUR),
            ram_style: Style::default().fg(FIRST_COLOUR),
            #[cfg(not(target_os = "windows"))]
            cache_style: Style::default().fg(FIFTH_COLOUR),
            swap_style: Style::default().fg(SECOND_COLOUR),
            arc_style: Style::default().fg(THIRD_COLOUR),
            gpu_colour_styles: vec![
                Style::default().fg(FOURTH_COLOUR),
                Style::default().fg(Color::LightBlue),
                Style::default().fg(Color::LightRed),
                Style::default().fg(Color::Cyan),
                Style::default().fg(Color::Green),
                Style::default().fg(Color::Blue),
                Style::default().fg(Color::Red),
            ],
            rx_style: Style::default().fg(FIRST_COLOUR),
            tx_style: Style::default().fg(SECOND_COLOUR),
            total_rx_style: Style::default().fg(THIRD_COLOUR),
            total_tx_style: Style::default().fg(FOURTH_COLOUR),
            all_colour_style: Style::default().fg(ALL_COLOUR),
            avg_colour_style: Style::default().fg(AVG_COLOUR),
            cpu_colour_styles: vec![
                Style::default().fg(Color::LightMagenta),
                Style::default().fg(Color::LightYellow),
                Style::default().fg(Color::LightCyan),
                Style::default().fg(Color::LightGreen),
                Style::default().fg(Color::LightBlue),
                Style::default().fg(Color::Cyan),
                Style::default().fg(Color::Green),
                Style::default().fg(Color::Blue),
            ],
            border_style: Style::default().fg(text_colour),
            highlighted_border_style: Style::default().fg(HIGHLIGHT_COLOUR),
            text_style: Style::default().fg(text_colour),
            widget_title_style: Style::default().fg(text_colour),
            graph_style: Style::default().fg(text_colour),
            high_battery_colour: Style::default().fg(Color::Green),
            medium_battery_colour: Style::default().fg(Color::Yellow),
            low_battery_colour: Style::default().fg(Color::Red),
            invalid_query_style: Style::default().fg(Color::Red),
            disabled_text_style: Style::default().fg(Color::DarkGray),
        }
    }
}

macro_rules! try_set_colour {
    ($field:expr, $colours:expr, $colour_field:ident) => {
        if let Some(colour_str) = &$colours.$colour_field {
            $field = str_to_fg(colour_str).map_err(|err| {
                OptionError::config(format!(
                    "Please update 'colors.{}' in your config file. {err}",
                    stringify!($colour_field)
                ))
            })?;
        }
    };
}

macro_rules! try_set_colour_list {
    ($field:expr, $colours:expr, $colour_field:ident) => {
        if let Some(colour_list) = &$colours.$colour_field {
            $field = colour_list
                .iter()
                .map(|s| str_to_fg(s))
                .collect::<Result<Vec<Style>, String>>()
                .map_err(|err| {
                    OptionError::config(format!(
                        "Please update 'colors.{}' in your config file. {err}",
                        stringify!($colour_field)
                    ))
                })?;
        }
    };
}

impl CanvasStyles {
    pub fn new(colour_scheme: ColourScheme, config: &ConfigV1) -> anyhow::Result<Self> {
        let mut canvas_colours = Self::default();

        match colour_scheme {
            ColourScheme::Default => {}
            ColourScheme::DefaultLight => {
                canvas_colours.set_colours_from_palette(&default_light_mode_colour_palette())?;
            }
            ColourScheme::Gruvbox => {
                canvas_colours.set_colours_from_palette(&gruvbox_colour_palette())?;
            }
            ColourScheme::GruvboxLight => {
                canvas_colours.set_colours_from_palette(&gruvbox_light_colour_palette())?;
            }
            ColourScheme::Nord => {
                canvas_colours.set_colours_from_palette(&nord_colour_palette())?;
            }
            ColourScheme::NordLight => {
                canvas_colours.set_colours_from_palette(&nord_light_colour_palette())?;
            }
            ColourScheme::Custom => {
                if let Some(colors) = &config.colors {
                    canvas_colours.set_colours_from_palette(colors)?;
                }
            }
        }

        Ok(canvas_colours)
    }

    pub fn set_colours_from_palette(&mut self, colours: &ColoursConfig) -> OptionResult<()> {
        // CPU
        try_set_colour!(self.avg_colour_style, colours, avg_cpu_color);
        try_set_colour!(self.all_colour_style, colours, all_cpu_color);
        try_set_colour_list!(self.cpu_colour_styles, colours, cpu_core_colors);

        // Memory
        #[cfg(not(target_os = "windows"))]
        try_set_colour!(self.cache_style, colours, cache_color);

        #[cfg(feature = "zfs")]
        try_set_colour!(self.arc_style, colours, arc_color);

        #[cfg(feature = "gpu")]
        try_set_colour_list!(self.gpu_colour_styles, colours, gpu_core_colors);

        try_set_colour!(self.ram_style, colours, ram_color);
        try_set_colour!(self.swap_style, colours, swap_color);

        // Network
        try_set_colour!(self.rx_style, colours, rx_color);
        try_set_colour!(self.tx_style, colours, tx_color);
        try_set_colour!(self.total_rx_style, colours, rx_total_color);
        try_set_colour!(self.total_tx_style, colours, tx_total_color);

        // Battery
        try_set_colour!(self.high_battery_colour, colours, high_battery_color);
        try_set_colour!(self.medium_battery_colour, colours, medium_battery_color);
        try_set_colour!(self.low_battery_colour, colours, low_battery_color);

        // Widget text and graphs
        try_set_colour!(self.widget_title_style, colours, widget_title_color);
        try_set_colour!(self.graph_style, colours, graph_color);
        try_set_colour!(self.text_style, colours, text_color);
        try_set_colour!(self.disabled_text_style, colours, disabled_text_color);
        try_set_colour!(self.border_style, colours, border_color);
        try_set_colour!(
            self.highlighted_border_style,
            colours,
            highlighted_border_color
        );

        // Tables
        try_set_colour!(self.table_header_style, colours, table_header_color);

        if let Some(scroll_entry_text_color) = &colours.selected_text_color {
            self.set_selected_text_fg(scroll_entry_text_color)
                .map_err(|err| {
                    OptionError::config(format!(
                        "Please update 'colors.selected_text_color' in your config file. {err}",
                    ))
                })?
        }

        if let Some(scroll_entry_bg_color) = &colours.selected_bg_color {
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

#[cfg(test)]
mod test {
    use tui::style::{Color, Style};

    use super::{CanvasStyles, ColourScheme};
    use crate::options::ConfigV1;

    #[test]
    fn default_selected_colour_works() {
        let mut colours = CanvasStyles::default();
        let original_selected_text_colour = CanvasStyles::DEFAULT_SELECTED_TEXT_STYLE.fg.unwrap();
        let original_selected_bg_colour = CanvasStyles::DEFAULT_SELECTED_TEXT_STYLE.bg.unwrap();

        assert_eq!(
            colours.selected_text_style,
            Style::default()
                .fg(original_selected_text_colour)
                .bg(original_selected_bg_colour),
        );

        colours.set_selected_text_fg("red").unwrap();
        assert_eq!(
            colours.selected_text_style,
            Style::default()
                .fg(Color::Red)
                .bg(original_selected_bg_colour),
        );

        colours.set_selected_text_bg("magenta").unwrap();
        assert_eq!(
            colours.selected_text_style,
            Style::default().fg(Color::Red).bg(Color::Magenta),
        );

        colours.set_selected_text_fg("fake blue").unwrap_err();
        assert_eq!(
            colours.selected_text_style,
            Style::default().fg(Color::Red).bg(Color::Magenta),
        );

        colours.set_selected_text_bg("fake blue").unwrap_err();
        assert_eq!(
            colours.selected_text_style,
            Style::default().fg(Color::Red).bg(Color::Magenta),
        );
    }

    #[test]
    fn built_in_colour_schemes_work() {
        let config = ConfigV1::default();
        CanvasStyles::new(ColourScheme::Default, &config).unwrap();
        CanvasStyles::new(ColourScheme::DefaultLight, &config).unwrap();
        CanvasStyles::new(ColourScheme::Gruvbox, &config).unwrap();
        CanvasStyles::new(ColourScheme::GruvboxLight, &config).unwrap();
        CanvasStyles::new(ColourScheme::Nord, &config).unwrap();
        CanvasStyles::new(ColourScheme::NordLight, &config).unwrap();
    }
}
