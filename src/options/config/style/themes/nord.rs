use tui::{
    style::{Color, Modifier},
    widgets::BorderType,
};

use super::{color, hex};
use crate::options::config::style::{Styles, utils::convert_hex_to_color};

impl Styles {
    pub(crate) fn nord_palette() -> Self {
        Self {
            ram_style: hex!("#88c0d0"),
            #[cfg(not(target_os = "windows"))]
            cache_style: hex!("#d8dee9"),
            swap_style: hex!("#d08770"),
            #[cfg(feature = "zfs")]
            arc_style: hex!("#5e81ac"),
            #[cfg(feature = "gpu")]
            gpu_colours: vec![
                hex!("#8fbcbb"),
                hex!("#81a1c1"),
                hex!("#d8dee9"),
                hex!("#b48ead"),
                hex!("#a3be8c"),
                hex!("#ebcb8b"),
                hex!("#bf616a"),
            ],
            rx_style: hex!("#88c0d0"),
            tx_style: hex!("#d08770"),
            total_rx_style: hex!("#5e81ac"),
            total_tx_style: hex!("#8fbcbb"),
            all_cpu_colour: hex!("#88c0d0"),
            avg_cpu_colour: hex!("#8fbcbb"),
            cpu_colour_styles: vec![
                hex!("#5e81ac"),
                hex!("#81a1c1"),
                hex!("#d8dee9"),
                hex!("#b48ead"),
                hex!("#a3be8c"),
                hex!("#ebcb8b"),
                hex!("#d08770"),
                hex!("#bf616a"),
            ],
            border_style: hex!("#88c0d0"),
            highlighted_border_style: hex!("#5e81ac"),
            text_style: hex!("#e5e9f0"),
            selected_text_style: hex!("#2e3440").bg(convert_hex_to_color("#88c0d0").unwrap()),
            table_header_style: hex!("#81a1c1").add_modifier(Modifier::BOLD),
            widget_title_style: hex!("#e5e9f0"),
            graph_style: hex!("#e5e9f0"),
            graph_legend_style: hex!("#e5e9f0"),
            high_battery: hex!("#a3be8c"),
            medium_battery: hex!("#ebcb8b"),
            low_battery: hex!("#bf616a"),
            invalid_query_style: color!(Color::Red),
            disabled_text_style: hex!("#4c566a"),
            border_type: BorderType::Plain,
            #[cfg(target_os = "linux")]
            thread_text_style: hex!("#a3be8c"),
        }
    }

    pub(crate) fn nord_light_palette() -> Self {
        Self {
            ram_style: hex!("#81a1c1"),
            #[cfg(not(target_os = "windows"))]
            cache_style: hex!("#4c566a"),
            swap_style: hex!("#d08770"),
            #[cfg(feature = "zfs")]
            arc_style: hex!("#5e81ac"),
            #[cfg(feature = "gpu")]
            gpu_colours: vec![
                hex!("#8fbcbb"),
                hex!("#88c0d0"),
                hex!("#4c566a"),
                hex!("#b48ead"),
                hex!("#a3be8c"),
                hex!("#ebcb8b"),
                hex!("#bf616a"),
            ],
            rx_style: hex!("#81a1c1"),
            tx_style: hex!("#d08770"),
            total_rx_style: hex!("#5e81ac"),
            total_tx_style: hex!("#8fbcbb"),
            all_cpu_colour: hex!("#81a1c1"),
            avg_cpu_colour: hex!("#8fbcbb"),
            cpu_colour_styles: vec![
                hex!("#5e81ac"),
                hex!("#88c0d0"),
                hex!("#4c566a"),
                hex!("#b48ead"),
                hex!("#a3be8c"),
                hex!("#ebcb8b"),
                hex!("#d08770"),
                hex!("#bf616a"),
            ],
            border_style: hex!("#2e3440"),
            highlighted_border_style: hex!("#5e81ac"),
            text_style: hex!("#2e3440"),
            selected_text_style: hex!("#f5f5f5").bg(convert_hex_to_color("#5e81ac").unwrap()),
            table_header_style: hex!("#5e81ac").add_modifier(Modifier::BOLD),
            widget_title_style: hex!("#2e3440"),
            graph_style: hex!("#2e3440"),
            graph_legend_style: hex!("#2e3440"),
            high_battery: hex!("#a3be8c"),
            medium_battery: hex!("#ebcb8b"),
            low_battery: hex!("#bf616a"),
            invalid_query_style: color!(Color::Red),
            disabled_text_style: hex!("#d8dee9"),
            border_type: BorderType::Plain,
            #[cfg(target_os = "linux")]
            thread_text_style: hex!("#a3be8c"),
        }
    }
}
