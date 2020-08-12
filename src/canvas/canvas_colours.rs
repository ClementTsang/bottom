use tui::style::{Color, Modifier, Style};

use colour_utils::*;

use crate::{constants::*, utils::error};

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
    // Full, Medium, Low
    pub battery_bar_styles: Vec<Style>,
    pub invalid_query_style: Style,
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
            cpu_colour_styles: Vec::new(),
            border_style: Style::default().fg(text_colour),
            highlighted_border_style: Style::default().fg(STANDARD_HIGHLIGHT_COLOUR),
            text_style: Style::default().fg(text_colour),
            widget_title_style: Style::default().fg(text_colour),
            graph_style: Style::default().fg(text_colour),
            battery_bar_styles: vec![
                Style::default().fg(Color::Red),
                Style::default().fg(Color::Yellow),
                Style::default().fg(Color::Yellow),
                Style::default().fg(Color::Green),
                Style::default().fg(Color::Green),
                Style::default().fg(Color::Green),
            ],
            invalid_query_style: tui::style::Style::default().fg(tui::style::Color::Red),
        }
    }
}

impl CanvasColours {
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
        self.table_header_style = get_style_from_config(colour)?.add_modifier(Modifier::BOLD);
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

    // pub fn set_rx_total_colour(&mut self, colour: &str) -> error::Result<()> {
    //     self.total_rx_style = get_style_from_config(colour)?;
    //     Ok(())
    // }

    // pub fn set_tx_total_colour(&mut self, colour: &str) -> error::Result<()> {
    //     self.total_tx_style = get_style_from_config(colour)?;
    //     Ok(())
    // }

    pub fn set_avg_cpu_colour(&mut self, colour: &str) -> error::Result<()> {
        self.avg_colour_style = get_style_from_config(colour)?;
        Ok(())
    }

    pub fn set_all_cpu_colour(&mut self, colour: &str) -> error::Result<()> {
        self.all_colour_style = get_style_from_config(colour)?;
        Ok(())
    }

    pub fn set_cpu_colours(&mut self, colours: &[String]) -> error::Result<()> {
        let max_amount = std::cmp::min(colours.len(), NUM_COLOURS);
        for (itx, colour) in colours.iter().enumerate() {
            if itx >= max_amount {
                break;
            }
            self.cpu_colour_styles.push(get_style_from_config(colour)?);
        }
        Ok(())
    }

    pub fn generate_remaining_cpu_colours(&mut self) {
        let remaining_num_colours = NUM_COLOURS.saturating_sub(self.cpu_colour_styles.len());
        self.cpu_colour_styles
            .extend(gen_n_styles(remaining_num_colours));
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

    pub fn set_battery_colours(&mut self, colours: &[String]) -> error::Result<()> {
        if colours.is_empty() {
            Err(error::BottomError::ConfigError(
                "invalid colour config: battery colour list must have at least one colour!"
                    .to_string(),
            ))
        } else {
            let generated_colours: Result<Vec<_>, _> = colours
                .iter()
                .map(|colour| get_style_from_config(colour))
                .collect();

            self.battery_bar_styles = generated_colours?;
            Ok(())
        }
    }
}
