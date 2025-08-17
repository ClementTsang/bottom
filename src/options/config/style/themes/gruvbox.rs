use tui::{
    style::{Color, Modifier},
    widgets::BorderType,
};

use super::{color, hex};
use crate::options::config::style::{Styles, utils::convert_hex_to_color};

impl Styles {
    pub(crate) fn gruvbox_palette() -> Self {
        Self {
            ram_style: hex!("#8ec07c"),
            #[cfg(not(target_os = "windows"))]
            cache_style: hex!("#b16286"),
            swap_style: hex!("#fabd2f"),
            #[cfg(feature = "zfs")]
            arc_style: hex!("#689d6a"),
            #[cfg(feature = "gpu")]
            gpu_colours: vec![
                hex!("#d79921"),
                hex!("#458588"),
                hex!("#b16286"),
                hex!("#fe8019"),
                hex!("#b8bb26"),
                hex!("#cc241d"),
                hex!("#98971a"),
            ],
            rx_style: hex!("#8ec07c"),
            tx_style: hex!("#fabd2f"),
            total_rx_style: hex!("#689d6a"),
            total_tx_style: hex!("#d79921"),
            all_cpu_colour: hex!("#8ec07c"),
            avg_cpu_colour: hex!("#fb4934"),
            cpu_colour_styles: vec![
                hex!("#cc241d"),
                hex!("#98971a"),
                hex!("#d79921"),
                hex!("#458588"),
                hex!("#b16286"),
                hex!("#689d6a"),
                hex!("#fe8019"),
                hex!("#b8bb26"),
                hex!("#fabd2f"),
                hex!("#83a598"),
                hex!("#d3869b"),
                hex!("#d65d0e"),
                hex!("#9d0006"),
                hex!("#79740e"),
                hex!("#b57614"),
                hex!("#076678"),
                hex!("#8f3f71"),
                hex!("#427b58"),
                hex!("#d65d03"),
                hex!("#af3a03"),
            ],
            border_style: hex!("#ebdbb2"),
            highlighted_border_style: hex!("#fe8019"),
            text_style: hex!("#ebdbb2"),
            selected_text_style: hex!("#1d2021").bg(convert_hex_to_color("#ebdbb2").unwrap()),
            table_header_style: hex!("#83a598").add_modifier(Modifier::BOLD),
            widget_title_style: hex!("#ebdbb2"),
            graph_style: hex!("#ebdbb2"),
            graph_legend_style: hex!("#ebdbb2"),
            high_battery: hex!("#98971a"),
            medium_battery: hex!("#fabd2f"),
            low_battery: hex!("#fb4934"),
            invalid_query_style: color!(Color::Red),
            disabled_text_style: hex!("#665c54"),
            border_type: BorderType::Plain,
            #[cfg(target_os = "linux")]
            thread_text_style: hex!("#458588"),
        }
    }

    pub(crate) fn gruvbox_light_palette() -> Self {
        Self {
            ram_style: hex!("#427b58"),
            #[cfg(not(target_os = "windows"))]
            cache_style: hex!("#d79921"),
            swap_style: hex!("#cc241d"),
            #[cfg(feature = "zfs")]
            arc_style: hex!("#689d6a"),
            #[cfg(feature = "gpu")]
            gpu_colours: vec![
                hex!("#9d0006"),
                hex!("#98971a"),
                hex!("#d79921"),
                hex!("#458588"),
                hex!("#b16286"),
                hex!("#fe8019"),
                hex!("#b8bb26"),
            ],
            rx_style: hex!("#427b58"),
            tx_style: hex!("#cc241d"),
            total_rx_style: hex!("#689d6a"),
            total_tx_style: hex!("#d79921"),
            all_cpu_colour: hex!("#8ec07c"),
            avg_cpu_colour: hex!("#fb4934"),
            cpu_colour_styles: vec![
                hex!("#cc241d"),
                hex!("#98971a"),
                hex!("#d79921"),
                hex!("#458588"),
                hex!("#b16286"),
                hex!("#689d6a"),
                hex!("#fe8019"),
                hex!("#b8bb26"),
                hex!("#fabd2f"),
                hex!("#83a598"),
                hex!("#d3869b"),
                hex!("#d65d0e"),
                hex!("#9d0006"),
                hex!("#79740e"),
                hex!("#b57614"),
                hex!("#076678"),
                hex!("#8f3f71"),
                hex!("#427b58"),
                hex!("#d65d03"),
                hex!("#af3a03"),
            ],
            border_style: hex!("#3c3836"),
            highlighted_border_style: hex!("#af3a03"),
            text_style: hex!("#3c3836"),
            selected_text_style: hex!("#ebdbb2").bg(convert_hex_to_color("#3c3836").unwrap()),
            table_header_style: hex!("#076678").add_modifier(Modifier::BOLD),
            widget_title_style: hex!("#3c3836"),
            graph_style: hex!("#3c3836"),
            graph_legend_style: hex!("#3c3836"),
            high_battery: hex!("#98971a"),
            medium_battery: hex!("#d79921"),
            low_battery: hex!("#cc241d"),
            invalid_query_style: color!(Color::Red),
            disabled_text_style: hex!("#d5c4a1"),
            border_type: BorderType::Plain,
            #[cfg(target_os = "linux")]
            thread_text_style: hex!("#458588"),
        }
    }
}
