use anyhow::Context;
use colour_utils::*;
use tui::style::{Color, Style};

use super::ColourScheme;
use crate::{
    constants::*,
    options::{Config, ConfigColours},
    utils::error,
};
mod colour_utils;

pub struct CanvasStyling {
    pub currently_selected_text_colour: Color,
    pub currently_selected_bg_colour: Color,
    pub currently_selected_text_style: Style,
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

impl Default for CanvasStyling {
    fn default() -> Self {
        let text_colour = Color::Gray;
        let currently_selected_text_colour = Color::Black;
        let currently_selected_bg_colour = HIGHLIGHT_COLOUR;

        CanvasStyling {
            currently_selected_text_colour,
            currently_selected_bg_colour,
            currently_selected_text_style: Style::default()
                .fg(currently_selected_text_colour)
                .bg(currently_selected_bg_colour),
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
            $field = str_to_fg(colour_str).context(concat!(
                "update '",
                stringify!($colour_field),
                "' in your config file"
            ))?;
        }
    };
}

macro_rules! try_set_colour_list {
    ($field:expr, $colours:expr, $colour_field:ident) => {
        if let Some(colour_list) = &$colours.$colour_field {
            $field = colour_list
                .iter()
                .map(|s| str_to_fg(s))
                .collect::<error::Result<Vec<Style>>>()
                .context(concat!(
                    "update '",
                    stringify!($colour_field),
                    "' in your config file"
                ))?;
        }
    };
}

impl CanvasStyling {
    pub fn new(colour_scheme: ColourScheme, config: &Config) -> anyhow::Result<Self> {
        let mut canvas_colours = Self::default();

        match colour_scheme {
            ColourScheme::Default => {}
            ColourScheme::DefaultLight => {
                canvas_colours.set_colours_from_palette(&DEFAULT_LIGHT_MODE_COLOUR_PALETTE)?;
            }
            ColourScheme::Gruvbox => {
                canvas_colours.set_colours_from_palette(&GRUVBOX_COLOUR_PALETTE)?;
            }
            ColourScheme::GruvboxLight => {
                canvas_colours.set_colours_from_palette(&GRUVBOX_LIGHT_COLOUR_PALETTE)?;
            }
            ColourScheme::Nord => {
                canvas_colours.set_colours_from_palette(&NORD_COLOUR_PALETTE)?;
            }
            ColourScheme::NordLight => {
                canvas_colours.set_colours_from_palette(&NORD_LIGHT_COLOUR_PALETTE)?;
            }
            ColourScheme::Custom => {
                if let Some(colors) = &config.colors {
                    canvas_colours.set_colours_from_palette(colors)?;
                }
            }
        }

        Ok(canvas_colours)
    }

    pub fn set_colours_from_palette(&mut self, colours: &ConfigColours) -> anyhow::Result<()> {
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
            self.set_scroll_entry_text_color(scroll_entry_text_color)
                .context("update 'selected_text_color' in your config file")?;
        }

        if let Some(scroll_entry_bg_color) = &colours.selected_bg_color {
            self.set_scroll_entry_bg_color(scroll_entry_bg_color)
                .context("update 'selected_bg_color' in your config file")?;
        }

        Ok(())
    }

    fn set_scroll_entry_text_color(&mut self, colour: &str) -> error::Result<()> {
        self.currently_selected_text_colour = str_to_colour(colour)?;
        self.currently_selected_text_style = Style::default()
            .fg(self.currently_selected_text_colour)
            .bg(self.currently_selected_bg_colour);

        Ok(())
    }

    fn set_scroll_entry_bg_color(&mut self, colour: &str) -> error::Result<()> {
        self.currently_selected_bg_colour = str_to_colour(colour)?;
        self.currently_selected_text_style = Style::default()
            .fg(self.currently_selected_text_colour)
            .bg(self.currently_selected_bg_colour);

        Ok(())
    }
}

#[cfg(test)]
mod test {
    use super::{CanvasStyling, ColourScheme};
    use crate::Config;
    use tui::style::{Color, Style};

    #[test]
    fn default_selected_colour_works() {
        let mut colours = CanvasStyling::default();

        assert_eq!(
            colours.currently_selected_text_style,
            Style::default()
                .fg(colours.currently_selected_text_colour)
                .bg(colours.currently_selected_bg_colour),
        );

        colours.set_scroll_entry_text_color("red").unwrap();
        assert_eq!(
            colours.currently_selected_text_style,
            Style::default()
                .fg(Color::Red)
                .bg(colours.currently_selected_bg_colour),
        );

        colours.set_scroll_entry_bg_color("magenta").unwrap();
        assert_eq!(
            colours.currently_selected_text_style,
            Style::default().fg(Color::Red).bg(Color::Magenta),
        );

        colours.set_scroll_entry_text_color("fake red").unwrap_err();
        assert_eq!(
            colours.currently_selected_text_style,
            Style::default()
                .fg(Color::Red)
                .bg(colours.currently_selected_bg_colour),
        );

        colours
            .set_scroll_entry_bg_color("fake magenta")
            .unwrap_err();
        assert_eq!(
            colours.currently_selected_text_style,
            Style::default().fg(Color::Red).bg(Color::Magenta),
        );
    }

    #[test]
    fn built_in_colour_schemes_work() {
        let config = Config::default();
        CanvasStyling::new(ColourScheme::Default, &config).unwrap();
        CanvasStyling::new(ColourScheme::DefaultLight, &config).unwrap();
        CanvasStyling::new(ColourScheme::Gruvbox, &config).unwrap();
        CanvasStyling::new(ColourScheme::GruvboxLight, &config).unwrap();
        CanvasStyling::new(ColourScheme::Nord, &config).unwrap();
        CanvasStyling::new(ColourScheme::NordLight, &config).unwrap();
    }
}
