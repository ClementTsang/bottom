use tui::{
    style::{Color, Modifier, Style},
    widgets::BorderType,
};

use super::color;
use crate::options::config::style::Styles;

impl Styles {
    pub(crate) fn default_style() -> Self {
        const FIRST_COLOUR: Color = Color::LightMagenta;
        const SECOND_COLOUR: Color = Color::LightYellow;
        const THIRD_COLOUR: Color = Color::LightCyan;
        const FOURTH_COLOUR: Color = Color::LightGreen;
        #[cfg(not(target_os = "windows"))]
        const FIFTH_COLOUR: Color = Color::LightRed;
        const HIGHLIGHT_COLOUR: Color = Color::LightBlue;
        const AVG_COLOUR: Color = Color::Red;
        const ALL_COLOUR: Color = Color::Green;
        const DEFAULT_SELECTED_TEXT_STYLE: Style = color!(Color::Black).bg(HIGHLIGHT_COLOUR);
        const TEXT_COLOUR: Color = Color::Gray;

        Self {
            ram_style: color!(FIRST_COLOUR),
            #[cfg(not(target_os = "windows"))]
            cache_style: color!(FIFTH_COLOUR),
            swap_style: color!(SECOND_COLOUR),
            #[cfg(feature = "zfs")]
            arc_style: color!(THIRD_COLOUR),
            #[cfg(feature = "gpu")]
            gpu_colours: vec![
                color!(FOURTH_COLOUR),
                color!(Color::LightBlue),
                color!(Color::LightRed),
                color!(Color::Cyan),
                color!(Color::Green),
                color!(Color::Blue),
                color!(Color::Red),
            ],
            rx_style: color!(FIRST_COLOUR),
            tx_style: color!(SECOND_COLOUR),
            total_rx_style: color!(THIRD_COLOUR),
            total_tx_style: color!(FOURTH_COLOUR),
            all_cpu_colour: color!(ALL_COLOUR),
            avg_cpu_colour: color!(AVG_COLOUR),
            cpu_colour_styles: vec![
                color!(Color::LightMagenta),
                color!(Color::LightYellow),
                color!(Color::LightCyan),
                color!(Color::LightGreen),
                color!(Color::LightBlue),
                color!(Color::Cyan),
                color!(Color::Green),
                color!(Color::Blue),
            ],
            border_style: color!(TEXT_COLOUR),
            highlighted_border_style: color!(HIGHLIGHT_COLOUR),
            text_style: color!(TEXT_COLOUR),
            selected_text_style: DEFAULT_SELECTED_TEXT_STYLE,
            table_header_style: color!(HIGHLIGHT_COLOUR).add_modifier(Modifier::BOLD),
            widget_title_style: color!(TEXT_COLOUR),
            graph_style: color!(TEXT_COLOUR),
            graph_legend_style: color!(TEXT_COLOUR),
            high_battery: color!(Color::Green),
            medium_battery: color!(Color::Yellow),
            low_battery: color!(Color::Red),
            invalid_query_style: color!(Color::Red),
            disabled_text_style: color!(Color::DarkGray),
            border_type: BorderType::Plain,
            #[cfg(target_os = "linux")]
            thread_text_style: color!(Color::Green),
        }
    }

    pub fn default_light_mode() -> Self {
        Self {
            ram_style: color!(Color::Blue),
            #[cfg(not(target_os = "windows"))]
            cache_style: color!(Color::LightRed),
            swap_style: color!(Color::Red),
            #[cfg(feature = "zfs")]
            arc_style: color!(Color::LightBlue),
            #[cfg(feature = "gpu")]
            gpu_colours: vec![
                color!(Color::LightGreen),
                color!(Color::LightCyan),
                color!(Color::LightRed),
                color!(Color::Cyan),
                color!(Color::Green),
                color!(Color::Blue),
                color!(Color::Red),
            ],
            rx_style: color!(Color::Blue),
            tx_style: color!(Color::Red),
            total_rx_style: color!(Color::LightBlue),
            total_tx_style: color!(Color::LightRed),
            cpu_colour_styles: vec![
                color!(Color::LightMagenta),
                color!(Color::LightBlue),
                color!(Color::LightRed),
                color!(Color::Cyan),
                color!(Color::Green),
                color!(Color::Blue),
                color!(Color::Red),
            ],
            border_style: color!(Color::Black),
            text_style: color!(Color::Black),
            selected_text_style: color!(Color::White).bg(Color::LightBlue),
            table_header_style: color!(Color::Black).add_modifier(Modifier::BOLD),
            widget_title_style: color!(Color::Black),
            graph_style: color!(Color::Black),
            graph_legend_style: color!(Color::Black),
            disabled_text_style: color!(Color::Gray),
            ..Self::default_style()
        }
    }
}
