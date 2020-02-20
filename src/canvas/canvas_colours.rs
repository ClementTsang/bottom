mod colour_utils;
use colour_utils::*;
use tui::style::{Color, Modifier, Style};

use crate::{constants::*, utils::error};

const STANDARD_FIRST_COLOUR: Color = Color::LightMagenta;
const STANDARD_SECOND_COLOUR: Color = Color::LightYellow;

pub struct CanvasColours {
	pub currently_selected_text_colour: Color,
	pub currently_selected_bg_colour: Color,
	pub currently_selected_text_style: Style,
	pub table_header_style: Style,
	pub ram_style: Style,
	pub swap_style: Style,
	pub rx_style: Style,
	pub tx_style: Style,
	pub cpu_colour_styles: Vec<Style>,
	pub border_style: Style,
	pub highlighted_border_style: Style,
	pub text_style: Style,
	pub widget_title_style: Style,
	pub graph_style: Style,
}

impl Default for CanvasColours {
	fn default() -> Self {
		CanvasColours {
			currently_selected_text_colour: Color::Black,
			currently_selected_bg_colour: Color::Cyan,
			currently_selected_text_style: Style::default().fg(Color::Black).bg(Color::Cyan),
			table_header_style: Style::default()
				.fg(Color::LightBlue)
				.modifier(Modifier::BOLD),
			ram_style: Style::default().fg(STANDARD_FIRST_COLOUR),
			swap_style: Style::default().fg(STANDARD_SECOND_COLOUR),
			rx_style: Style::default().fg(STANDARD_FIRST_COLOUR),
			tx_style: Style::default().fg(STANDARD_SECOND_COLOUR),
			cpu_colour_styles: Vec::new(),
			border_style: Style::default().fg(Color::Gray),
			highlighted_border_style: Style::default().fg(Color::LightBlue),
			text_style: Style::default().fg(Color::Gray),
			widget_title_style: Style::default().fg(Color::Gray),
			graph_style: Style::default().fg(Color::Gray),
		}
	}
}

impl CanvasColours {
	pub fn set_text_colour(&mut self, hex: &str) -> error::Result<()> {
		self.text_style = Style::default().fg(convert_hex_to_color(hex)?);
		Ok(())
	}
	pub fn set_border_colour(&mut self, hex: &str) -> error::Result<()> {
		self.border_style = Style::default().fg(convert_hex_to_color(hex)?);
		Ok(())
	}
	pub fn set_highlighted_border_colour(&mut self, hex: &str) -> error::Result<()> {
		self.highlighted_border_style = Style::default().fg(convert_hex_to_color(hex)?);
		Ok(())
	}
	pub fn set_table_header_colour(&mut self, hex: &str) -> error::Result<()> {
		self.table_header_style = Style::default()
			.fg(convert_hex_to_color(hex)?)
			.modifier(Modifier::BOLD);
		Ok(())
	}
	pub fn set_ram_colour(&mut self, hex: &str) -> error::Result<()> {
		self.ram_style = Style::default().fg(convert_hex_to_color(hex)?);
		Ok(())
	}
	pub fn set_swap_colour(&mut self, hex: &str) -> error::Result<()> {
		self.swap_style = Style::default().fg(convert_hex_to_color(hex)?);
		Ok(())
	}
	pub fn set_rx_colour(&mut self, hex: &str) -> error::Result<()> {
		self.rx_style = Style::default().fg(convert_hex_to_color(hex)?);
		Ok(())
	}
	pub fn set_tx_colour(&mut self, hex: &str) -> error::Result<()> {
		self.tx_style = Style::default().fg(convert_hex_to_color(hex)?);
		Ok(())
	}
	pub fn set_cpu_colours(&mut self, hex_colours: &[String]) -> error::Result<()> {
		let max_amount = std::cmp::min(hex_colours.len(), NUM_COLOURS as usize);
		for (itx, hex_colour) in hex_colours.iter().enumerate() {
			if itx >= max_amount {
				break;
			}
			self.cpu_colour_styles
				.push(Style::default().fg(convert_hex_to_color(hex_colour)?));
		}
		Ok(())
	}
	pub fn generate_remaining_cpu_colours(&mut self) {
		let remaining_num_colours = NUM_COLOURS - self.cpu_colour_styles.len() as i32;
		self.cpu_colour_styles
			.extend(gen_n_styles(remaining_num_colours));
	}

	pub fn set_scroll_entry_text_color(&mut self, hex: &str) -> error::Result<()> {
		self.currently_selected_text_colour = convert_hex_to_color(hex)?;
		self.currently_selected_text_style = Style::default()
			.fg(self.currently_selected_text_colour)
			.bg(self.currently_selected_bg_colour);
		Ok(())
	}
	pub fn set_scroll_entry_bg_color(&mut self, hex: &str) -> error::Result<()> {
		self.currently_selected_bg_colour = convert_hex_to_color(hex)?;
		self.currently_selected_text_style = Style::default()
			.fg(self.currently_selected_text_colour)
			.bg(self.currently_selected_bg_colour);
		Ok(())
	}

	pub fn set_widget_title_colour(&mut self, hex: &str) -> error::Result<()> {
		self.widget_title_style = Style::default().fg(convert_hex_to_color(hex)?);
		Ok(())
	}

	pub fn set_graph_colour(&mut self, hex: &str) -> error::Result<()> {
		self.graph_style = Style::default().fg(convert_hex_to_color(hex)?);
		Ok(())
	}
}
