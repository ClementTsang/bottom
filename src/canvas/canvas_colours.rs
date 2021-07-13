use crate::{options::ConfigColours, utils::error};
use anyhow::Context;
use colour_utils::*;
use tui::style::{Color, Style};
mod colour_utils;

pub struct CanvasColours {
    pub currently_selected_text_colour: Color,
    pub currently_selected_bg_colour: Color,
    pub currently_selected_text_style: Style,
    pub table_header_style: Style,
    pub ram_style: Style,
    pub swap_style: Style,
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

impl Default for CanvasColours {
    fn default() -> Self {
        let text_colour = Color::Gray;

        CanvasColours {
            currently_selected_text_colour: Color::Black,
            currently_selected_bg_colour: Color::Cyan,
            currently_selected_text_style: Style::default()
                .fg(Color::Black)
                .bg(STANDARD_HIGHLIGHT_COLOUR),
            table_header_style: Style::default().fg(STANDARD_HIGHLIGHT_COLOUR),
            ram_style: Style::default().fg(STANDARD_FIRST_COLOUR),
            swap_style: Style::default().fg(STANDARD_SECOND_COLOUR),
            rx_style: Style::default().fg(STANDARD_FIRST_COLOUR),
            tx_style: Style::default().fg(STANDARD_SECOND_COLOUR),
            total_rx_style: Style::default().fg(STANDARD_THIRD_COLOUR),
            total_tx_style: Style::default().fg(STANDARD_FOURTH_COLOUR),
            all_colour_style: Style::default().fg(ALL_COLOUR),
            avg_colour_style: Style::default().fg(AVG_COLOUR),
            cpu_colour_styles: vec![
                Style::default().fg(Color::LightMagenta),
                Style::default().fg(Color::LightYellow),
                Style::default().fg(Color::LightCyan),
                Style::default().fg(Color::LightGreen),
                Style::default().fg(Color::LightBlue),
                Style::default().fg(Color::LightRed),
                Style::default().fg(Color::Cyan),
                Style::default().fg(Color::Green),
                Style::default().fg(Color::Blue),
                Style::default().fg(Color::Red),
            ],
            border_style: Style::default().fg(text_colour),
            highlighted_border_style: Style::default().fg(STANDARD_HIGHLIGHT_COLOUR),
            text_style: Style::default().fg(text_colour),
            widget_title_style: Style::default().fg(text_colour),
            graph_style: Style::default().fg(text_colour),
            high_battery_colour: Style::default().fg(Color::Green),
            medium_battery_colour: Style::default().fg(Color::Yellow),
            low_battery_colour: Style::default().fg(Color::Red),
            invalid_query_style: Style::default().fg(tui::style::Color::Red),
            disabled_text_style: Style::default().fg(Color::DarkGray),
        }
    }
}

impl CanvasColours {
    pub fn set_colours_from_palette(&mut self, colours: &ConfigColours) -> anyhow::Result<()> {
        if let Some(border_color) = &colours.border_color {
            self.set_border_colour(border_color)
                .context("Update 'border_color' in your config file.")?;
        }

        if let Some(highlighted_border_color) = &colours.highlighted_border_color {
            self.set_highlighted_border_colour(highlighted_border_color)
                .context("Update 'highlighted_border_color' in your config file.")?;
        }

        if let Some(text_color) = &colours.text_color {
            self.set_text_colour(text_color)
                .context("Update 'text_color' in your config file.")?;
        }

        if let Some(avg_cpu_color) = &colours.avg_cpu_color {
            self.set_avg_cpu_colour(avg_cpu_color)
                .context("Update 'avg_cpu_color' in your config file.")?;
        }

        if let Some(all_cpu_color) = &colours.all_cpu_color {
            self.set_all_cpu_colour(all_cpu_color)
                .context("Update 'all_cpu_color' in your config file.")?;
        }

        if let Some(cpu_core_colors) = &colours.cpu_core_colors {
            self.set_cpu_colours(cpu_core_colors)
                .context("Update 'cpu_core_colors' in your config file.")?;
        }

        if let Some(ram_color) = &colours.ram_color {
            self.set_ram_colour(ram_color)
                .context("Update 'ram_color' in your config file.")?;
        }

        if let Some(swap_color) = &colours.swap_color {
            self.set_swap_colour(swap_color)
                .context("Update 'swap_color' in your config file.")?;
        }

        if let Some(rx_color) = &colours.rx_color {
            self.set_rx_colour(rx_color)
                .context("Update 'rx_color' in your config file.")?;
        }

        if let Some(tx_color) = &colours.tx_color {
            self.set_tx_colour(tx_color)
                .context("Update 'tx_color' in your config file.")?;
        }

        if let Some(table_header_color) = &colours.table_header_color {
            self.set_table_header_colour(table_header_color)
                .context("Update 'table_header_color' in your config file.")?;
        }

        if let Some(scroll_entry_text_color) = &colours.selected_text_color {
            self.set_scroll_entry_text_color(scroll_entry_text_color)
                .context("Update 'selected_text_color' in your config file.")?;
        }

        if let Some(scroll_entry_bg_color) = &colours.selected_bg_color {
            self.set_scroll_entry_bg_color(scroll_entry_bg_color)
                .context("Update 'selected_bg_color' in your config file.")?;
        }

        if let Some(widget_title_color) = &colours.widget_title_color {
            self.set_widget_title_colour(widget_title_color)
                .context("Update 'widget_title_color' in your config file.")?;
        }

        if let Some(graph_color) = &colours.graph_color {
            self.set_graph_colour(graph_color)
                .context("Update 'graph_color' in your config file.")?;
        }

        if let Some(high_battery_color) = &colours.high_battery_color {
            self.set_high_battery_color(high_battery_color)
                .context("Update 'high_battery_color' in your config file.")?;
        }

        if let Some(medium_battery_color) = &colours.medium_battery_color {
            self.set_medium_battery_color(medium_battery_color)
                .context("Update 'medium_battery_color' in your config file.")?;
        }

        if let Some(low_battery_color) = &colours.low_battery_color {
            self.set_low_battery_color(low_battery_color)
                .context("Update 'low_battery_color' in your config file.")?;
        }

        if let Some(disabled_text_color) = &colours.disabled_text_color {
            self.set_disabled_text_colour(disabled_text_color)
                .context("Update 'disabled_text_color' in your config file.")?;
        }

        if let Some(rx_total_color) = &colours.rx_total_color {
            self.set_rx_total_colour(rx_total_color)?;
        }

        if let Some(tx_total_color) = &colours.tx_total_color {
            self.set_tx_total_colour(tx_total_color)?;
        }

        Ok(())
    }

    pub fn set_disabled_text_colour(&mut self, colour: &str) -> error::Result<()> {
        self.disabled_text_style = get_style_from_config(colour)?;
        Ok(())
    }

    pub fn set_text_colour(&mut self, colour: &str) -> error::Result<()> {
        self.text_style = get_style_from_config(colour)?;
        Ok(())
    }

    pub fn set_border_colour(&mut self, colour: &str) -> error::Result<()> {
        self.border_style = get_style_from_config(colour)?;
        Ok(())
    }

    pub fn set_highlighted_border_colour(&mut self, colour: &str) -> error::Result<()> {
        self.highlighted_border_style = get_style_from_config(colour)?;
        Ok(())
    }

    pub fn set_table_header_colour(&mut self, colour: &str) -> error::Result<()> {
        self.table_header_style = get_style_from_config(colour)?;
        // Disabled as it seems to be bugged when I go into full command mode...?  It becomes huge lol
        // self.table_header_style = get_style_from_config(colour)?.modifier(Modifier::BOLD);
        Ok(())
    }

    pub fn set_ram_colour(&mut self, colour: &str) -> error::Result<()> {
        self.ram_style = get_style_from_config(colour)?;
        Ok(())
    }

    pub fn set_swap_colour(&mut self, colour: &str) -> error::Result<()> {
        self.swap_style = get_style_from_config(colour)?;
        Ok(())
    }

    pub fn set_rx_colour(&mut self, colour: &str) -> error::Result<()> {
        self.rx_style = get_style_from_config(colour)?;
        Ok(())
    }

    pub fn set_tx_colour(&mut self, colour: &str) -> error::Result<()> {
        self.tx_style = get_style_from_config(colour)?;
        Ok(())
    }

    pub fn set_rx_total_colour(&mut self, colour: &str) -> error::Result<()> {
        self.total_rx_style = get_style_from_config(colour)?;
        Ok(())
    }

    pub fn set_tx_total_colour(&mut self, colour: &str) -> error::Result<()> {
        self.total_tx_style = get_style_from_config(colour)?;
        Ok(())
    }

    pub fn set_avg_cpu_colour(&mut self, colour: &str) -> error::Result<()> {
        self.avg_colour_style = get_style_from_config(colour)?;
        Ok(())
    }

    pub fn set_all_cpu_colour(&mut self, colour: &str) -> error::Result<()> {
        self.all_colour_style = get_style_from_config(colour)?;
        Ok(())
    }

    pub fn set_cpu_colours(&mut self, colours: &[String]) -> error::Result<()> {
        self.cpu_colour_styles = colours
            .iter()
            .map(|colour| get_style_from_config(colour))
            .collect::<error::Result<Vec<Style>>>()?;
        Ok(())
    }

    pub fn set_scroll_entry_text_color(&mut self, colour: &str) -> error::Result<()> {
        self.currently_selected_text_colour = get_colour_from_config(colour)?;
        self.currently_selected_text_style = Style::default()
            .fg(self.currently_selected_text_colour)
            .bg(self.currently_selected_bg_colour);
        Ok(())
    }

    pub fn set_scroll_entry_bg_color(&mut self, colour: &str) -> error::Result<()> {
        self.currently_selected_bg_colour = get_colour_from_config(colour)?;
        self.currently_selected_text_style = Style::default()
            .fg(self.currently_selected_text_colour)
            .bg(self.currently_selected_bg_colour);
        Ok(())
    }

    pub fn set_widget_title_colour(&mut self, colour: &str) -> error::Result<()> {
        self.widget_title_style = get_style_from_config(colour)?;
        Ok(())
    }

    pub fn set_graph_colour(&mut self, colour: &str) -> error::Result<()> {
        self.graph_style = get_style_from_config(colour)?;
        Ok(())
    }

    pub fn set_high_battery_color(&mut self, colour: &str) -> error::Result<()> {
        self.high_battery_colour = get_style_from_config(colour)?;
        Ok(())
    }

    pub fn set_medium_battery_color(&mut self, colour: &str) -> error::Result<()> {
        self.medium_battery_colour = get_style_from_config(colour)?;
        Ok(())
    }

    pub fn set_low_battery_color(&mut self, colour: &str) -> error::Result<()> {
        self.low_battery_colour = get_style_from_config(colour)?;
        Ok(())
    }
}
