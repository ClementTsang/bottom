use crate::{
	app::{self, data_harvester::processes::ProcessHarvest},
	constants::*,
	data_conversion::{ConvertedCpuData, ConvertedProcessData},
	utils::{error, gen_util::*},
};
use std::cmp::max;
use std::collections::HashMap;
use tui::{
	backend,
	layout::{Alignment, Constraint, Direction, Layout, Rect},
	style::{Color, Style},
	terminal::Frame,
	widgets::{Axis, Block, Borders, Chart, Dataset, Marker, Paragraph, Row, Table, Text, Widget},
	Terminal,
};

const STANDARD_FIRST_COLOUR: Color = Color::Rgb(150, 106, 253);
const STANDARD_SECOND_COLOUR: Color = Color::LightYellow;
const GOLDEN_RATIO: f32 = 0.618_034; // Approx, good enough for use (also Clippy gets mad if it's too long)

// Headers
const CPU_LEGEND_HEADER: [&str; 2] = ["CPU", "Use%"];
const DISK_HEADERS: [&str; 7] = ["Disk", "Mount", "Used", "Free", "Total", "R/s", "W/s"];
const TEMP_HEADERS: [&str; 2] = ["Sensor", "Temp"];
const MEM_HEADERS: [&str; 3] = ["Mem", "Usage", "Usage%"];
const NON_WINDOWS_NETWORK_HEADERS: [&str; 4] = ["RX", "TX", "Total RX", "Total TX"];
const WINDOWS_NETWORK_HEADERS: [&str; 4] = ["RX", "TX", "", ""];
const FORCE_MIN_THRESHOLD: usize = 5;

lazy_static! {
	static ref HELP_TEXT: [Text<'static>; 22] = [
		Text::raw("\nGeneral Keybindings\n"),
		Text::raw("q, Ctrl-c to quit.  Note if you are currently in the search widget, `q` will not work.\n"),
		Text::raw("Ctrl-r to reset all data.\n"),
		Text::raw("f to toggle freezing and unfreezing the display.\n"),
		Text::raw(
			"Ctrl/Shift-Up, Ctrl/Shift-Down, Ctrl/Shift-Left, and Ctrl/Shift-Right to navigate between widgets.\n"
		),
		Text::raw("Up or k and Down or j scrolls through a list.\n"),
		Text::raw("Esc to close a dialog window (help or dd confirmation).\n"),
		Text::raw("? to get this help screen.\n"),
		Text::raw("\n Process Widget Keybindings\n"),
		Text::raw("dd to kill the selected process.\n"),
		Text::raw("c to sort by CPU usage.\n"),
		Text::raw("m to sort by memory usage.\n"),
		Text::raw("p to sort by PID.\n"),
		Text::raw("n to sort by process name.\n"),
		Text::raw("Tab to group together processes with the same name.\n"),
		Text::raw("Ctrl-f to toggle searching for a process.  / to just open it.\n"),
		Text::raw("Use Ctrl-p and Ctrl-n to toggle between searching for PID and name.\n"),
		Text::raw("Use Tab to set the cursor to the start and end of the bar respectively.\n"),
		Text::raw("Use Alt-c to toggle whether to ignore case.\n"),
		Text::raw("Use Alt-m to toggle matching the entire word.\n"),
		Text::raw("Use Alt-r to toggle regex.\n"),
		Text::raw("\nFor startup flags, type in \"btm -h\".")
	];
	static ref DISK_HEADERS_LENS: Vec<usize> = DISK_HEADERS
		.iter()
		.map(|entry| max(FORCE_MIN_THRESHOLD, entry.len()))
		.collect::<Vec<_>>();
	static ref CPU_LEGEND_HEADER_LENS: Vec<usize> = CPU_LEGEND_HEADER
		.iter()
		.map(|entry| max(FORCE_MIN_THRESHOLD, entry.len()))
		.collect::<Vec<_>>();
	static ref TEMP_HEADERS_LENS: Vec<usize> = TEMP_HEADERS
		.iter()
		.map(|entry| max(FORCE_MIN_THRESHOLD, entry.len()))
		.collect::<Vec<_>>();
	static ref MEM_HEADERS_LENS: Vec<usize> = MEM_HEADERS
		.iter()
		.map(|entry| max(FORCE_MIN_THRESHOLD, entry.len()))
		.collect::<Vec<_>>();
	static ref NON_WINDOWS_NETWORK_HEADERS_LENS: Vec<usize> = NON_WINDOWS_NETWORK_HEADERS
		.iter()
		.map(|entry| max(FORCE_MIN_THRESHOLD, entry.len()))
		.collect::<Vec<_>>();
	static ref WINDOWS_NETWORK_HEADERS_LENS: Vec<usize> = WINDOWS_NETWORK_HEADERS
		.iter()
		.map(|entry| max(FORCE_MIN_THRESHOLD, entry.len()))
		.collect::<Vec<_>>();
}

#[derive(Default)]
pub struct DisplayableData {
	pub rx_display: String,
	pub tx_display: String,
	pub total_rx_display: String,
	pub total_tx_display: String,
	pub network_data_rx: Vec<(f64, f64)>,
	pub network_data_tx: Vec<(f64, f64)>,
	pub disk_data: Vec<Vec<String>>,
	pub temp_sensor_data: Vec<Vec<String>>,
	pub process_data: HashMap<u32, ProcessHarvest>, // Not the final value
	pub grouped_process_data: Vec<ConvertedProcessData>, // Not the final value
	pub finalized_process_data: Vec<ConvertedProcessData>, // What's actually displayed
	pub mem_label: String,
	pub swap_label: String,
	pub mem_data: Vec<(f64, f64)>,
	pub swap_data: Vec<(f64, f64)>,
	pub cpu_data: Vec<ConvertedCpuData>,
}

/// Generates random colours.  Strategy found from
/// https://martin.ankerl.com/2009/12/09/how-to-create-random-colors-programmatically/
fn gen_n_styles(num_to_gen: i32) -> Vec<Style> {
	fn gen_hsv(h: f32) -> f32 {
		let new_val = h + GOLDEN_RATIO;
		if new_val > 1.0 {
			new_val.fract()
		} else {
			new_val
		}
	}
	/// This takes in an h, s, and v value of range [0, 1]
	/// For explanation of what this does, see
	/// https://en.wikipedia.org/wiki/HSL_and_HSV#HSV_to_RGB_alternative
	fn hsv_to_rgb(hue: f32, saturation: f32, value: f32) -> (u8, u8, u8) {
		fn hsv_helper(num: u32, hu: f32, sat: f32, val: f32) -> f32 {
			let k = (num as f32 + hu * 6.0) % 6.0;
			val - val * sat * float_max(float_min(k, float_min(4.1 - k, 1.1)), 0.0)
		}

		(
			(hsv_helper(5, hue, saturation, value) * 255.0) as u8,
			(hsv_helper(3, hue, saturation, value) * 255.0) as u8,
			(hsv_helper(1, hue, saturation, value) * 255.0) as u8,
		)
	}

	// Generate colours
	let mut colour_vec: Vec<Style> = vec![
		Style::default().fg(Color::Rgb(150, 106, 253)),
		Style::default().fg(Color::LightYellow),
		Style::default().fg(Color::LightMagenta),
		Style::default().fg(Color::LightCyan),
		Style::default().fg(Color::Green),
		Style::default().fg(Color::Red),
	];

	let mut h: f32 = 0.4; // We don't need random colours... right?
	for _i in 0..(num_to_gen - 6) {
		h = gen_hsv(h);
		let result = hsv_to_rgb(h, 0.5, 0.95);
		colour_vec.push(Style::default().fg(Color::Rgb(result.0, result.1, result.2)));
	}

	colour_vec
}

fn convert_hex_to_color(hex: &str) -> error::Result<Color> {
	fn convert_hex_to_rgb(hex: &str) -> error::Result<(u8, u8, u8)> {
		if hex.len() == 7 && &hex[0..1] == "#" {
			let r = u8::from_str_radix(&hex[1..3], 16)?;
			let g = u8::from_str_radix(&hex[3..5], 16)?;
			let b = u8::from_str_radix(&hex[5..7], 16)?;

			return Ok((r, g, b));
		}

		Err(error::BottomError::GenericError {
			message: format!(
				"Colour hex {} is not of valid length.  It must be a 7 character string of the form \"#112233\".",
				hex
			),
		})
	}

	let rgb = convert_hex_to_rgb(hex)?;
	Ok(Color::Rgb(rgb.0, rgb.1, rgb.2))
}

#[allow(dead_code)]
#[derive(Default)]
/// Handles the canvas' state.  TODO: [OPT] implement this.
pub struct Painter {
	height: f64,
	width: f64,
	vertical_dialog_chunk: Vec<Rect>,
	middle_dialog_chunk: Vec<Rect>,
	vertical_chunks: Vec<Rect>,
	middle_chunks: Vec<Rect>,
	middle_divided_chunk_2: Vec<Rect>,
	bottom_chunks: Vec<Rect>,
	cpu_chunk: Vec<Rect>,
	network_chunk: Vec<Rect>,
	pub colours: CanvasColours,
}

pub struct CanvasColours {
	text_colour: Color,
	scroll_text_colour: Color,
	scroll_bg_colour: Color,
	currently_selected_text_style: Style,
	table_header_style: Style,
	ram_colour: Color,
	swap_colour: Color,
	rx_colour: Color,
	tx_colour: Color,
	cpu_colour_styles: Vec<Style>,
	border_style: Style,
	highlighted_border_style: Style,
	text_style: Style,
	widget_title_style: Style,
	graph_style: Style,
}

impl Default for CanvasColours {
	fn default() -> Self {
		CanvasColours {
			text_colour: Color::Gray,
			scroll_text_colour: Color::Black,
			scroll_bg_colour: Color::Cyan,
			currently_selected_text_style: Style::default().fg(Color::Black).bg(Color::Cyan),
			table_header_style: Style::default().fg(Color::LightBlue),
			ram_colour: STANDARD_FIRST_COLOUR,
			swap_colour: STANDARD_SECOND_COLOUR,
			rx_colour: STANDARD_FIRST_COLOUR,
			tx_colour: STANDARD_SECOND_COLOUR,
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
		self.text_colour = convert_hex_to_color(hex)?;
		self.text_style = Style::default().fg(self.text_colour);
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
		self.table_header_style = Style::default().fg(convert_hex_to_color(hex)?);
		Ok(())
	}
	pub fn set_ram_colour(&mut self, hex: &str) -> error::Result<()> {
		self.ram_colour = convert_hex_to_color(hex)?;
		Ok(())
	}
	pub fn set_swap_colour(&mut self, hex: &str) -> error::Result<()> {
		self.swap_colour = convert_hex_to_color(hex)?;
		Ok(())
	}
	pub fn set_rx_colour(&mut self, hex: &str) -> error::Result<()> {
		self.rx_colour = convert_hex_to_color(hex)?;
		Ok(())
	}
	pub fn set_tx_colour(&mut self, hex: &str) -> error::Result<()> {
		self.tx_colour = convert_hex_to_color(hex)?;
		Ok(())
	}
	pub fn set_cpu_colours(&mut self, hex_colours: &Vec<String>) -> error::Result<()> {
		let max_amount = std::cmp::min(hex_colours.len(), NUM_COLOURS as usize);
		for i in 0..max_amount {
			self.cpu_colour_styles
				.push(Style::default().fg(convert_hex_to_color(&hex_colours[i])?));
		}
		Ok(())
	}
	pub fn generate_remaining_cpu_colours(&mut self) {
		let remaining_num_colours = NUM_COLOURS - self.cpu_colour_styles.len() as i32;
		self.cpu_colour_styles
			.extend(gen_n_styles(remaining_num_colours));
	}

	pub fn set_scroll_entry_text_color(&mut self, hex: &str) -> error::Result<()> {
		self.scroll_text_colour = convert_hex_to_color(hex)?;
		self.currently_selected_text_style = Style::default()
			.fg(self.scroll_text_colour)
			.bg(self.scroll_bg_colour);
		Ok(())
	}
	pub fn set_scroll_entry_bg_color(&mut self, hex: &str) -> error::Result<()> {
		self.scroll_bg_colour = convert_hex_to_color(hex)?;
		self.currently_selected_text_style = Style::default()
			.fg(self.scroll_text_colour)
			.bg(self.scroll_bg_colour);
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

impl Painter {
	pub fn draw_data<B: backend::Backend>(
		&mut self, terminal: &mut Terminal<B>, app_state: &mut app::App,
	) -> error::Result<()> {
		terminal.autoresize()?;
		terminal.draw(|mut f| {
			if app_state.show_help {
				// Only for the help
				let vertical_dialog_chunk = Layout::default()
					.direction(Direction::Vertical)
					.margin(1)
					.constraints(
						[
							Constraint::Percentage(22),
							Constraint::Percentage(60),
							Constraint::Percentage(18),
						]
						.as_ref(),
					)
					.split(f.size());

				let middle_dialog_chunk = Layout::default()
					.direction(Direction::Horizontal)
					.margin(0)
					.constraints(
						[
							Constraint::Percentage(20),
							Constraint::Percentage(60),
							Constraint::Percentage(20),
						]
						.as_ref(),
					)
					.split(vertical_dialog_chunk[1]);

				Paragraph::new(HELP_TEXT.iter())
					.block(
						Block::default()
							.title(" Help (Press Esc to close) ")
							.title_style(self.colours.widget_title_style)
							.style(self.colours.border_style)
							.borders(Borders::ALL),
					)
					.style(Style::default().fg(self.colours.text_colour))
					.alignment(Alignment::Left)
					.wrap(true)
					.render(&mut f, middle_dialog_chunk[1]);
			} else if app_state.show_dd {
				let vertical_dialog_chunk = Layout::default()
					.direction(Direction::Vertical)
					.margin(1)
					.constraints(
						[
							Constraint::Percentage(40),
							Constraint::Percentage(20),
							Constraint::Percentage(40),
						]
						.as_ref(),
					)
					.split(f.size());

				let middle_dialog_chunk = Layout::default()
					.direction(Direction::Horizontal)
					.margin(0)
					.constraints(
						[
							Constraint::Percentage(30),
							Constraint::Percentage(40),
							Constraint::Percentage(30),
						]
						.as_ref(),
					)
					.split(vertical_dialog_chunk[1]);

				if let Some(dd_err) = app_state.dd_err.clone() {
					let dd_text = [Text::raw(format!(
						"\nFailure to properly kill the process - {}",
						dd_err
					))];

					Paragraph::new(dd_text.iter())
						.block(
							Block::default()
								.title(" Kill Process Error (Press Esc to close) ")
								.title_style(self.colours.text_style)
								.style(self.colours.border_style)
								.borders(Borders::ALL),
						)
						.style(Style::default().fg(self.colours.text_colour))
						.alignment(Alignment::Center)
						.wrap(true)
						.render(&mut f, middle_dialog_chunk[1]);
				} else if let Some(to_kill_processes) = app_state.get_to_delete_processes() {
					if let Some(first_pid) = to_kill_processes.1.first() {
						let dd_text = [
							if app_state.is_grouped() {
								Text::raw(format!(
									"\nAre you sure you want to kill {} process(es) with name {}?",
									to_kill_processes.1.len(), to_kill_processes.0
								))
							} else {
								Text::raw(format!(
									"\nAre you sure you want to kill process {} with PID {}?",
									to_kill_processes.0, first_pid
								))
							},
							Text::raw("\n\nPress ENTER to proceed, ESC to exit."),
							Text::raw("\nNote that if bottom is frozen, it must be unfrozen for changes to be shown."),
						];

						Paragraph::new(dd_text.iter())
							.block(
								Block::default()
									.title(" Kill Process Confirmation (Press Esc to close) ")
									.title_style(self.colours.widget_title_style)
									.style(self.colours.border_style)
									.borders(Borders::ALL),
							)
							.style(Style::default().fg(self.colours.text_colour))
							.alignment(Alignment::Center)
							.wrap(true)
							.render(&mut f, middle_dialog_chunk[1]);
					} else {
						// This is a bit nasty, but it works well... I guess.
						app_state.show_dd = false;
					}
				} else {
					// This is a bit nasty, but it works well... I guess.
					app_state.show_dd = false;
				}
			} else {
				// TODO: [TUI] Change this back to a more even 33/33/34 when TUI releases
				let vertical_chunks = Layout::default()
					.direction(Direction::Vertical)
					.margin(1)
					.constraints(
						[
							Constraint::Percentage(30),
							Constraint::Percentage(37),
							Constraint::Percentage(33),
						]
						.as_ref(),
					)
					.split(f.size());

				let middle_chunks = Layout::default()
					.direction(Direction::Horizontal)
					.margin(0)
					.constraints([Constraint::Percentage(60), Constraint::Percentage(40)].as_ref())
					.split(vertical_chunks[1]);

				let middle_divided_chunk_2 = Layout::default()
					.direction(Direction::Vertical)
					.margin(0)
					.constraints([Constraint::Percentage(50), Constraint::Percentage(50)].as_ref())
					.split(middle_chunks[1]);

				let bottom_chunks = Layout::default()
					.direction(Direction::Horizontal)
					.margin(0)
					.constraints([Constraint::Percentage(50), Constraint::Percentage(50)].as_ref())
					.split(vertical_chunks[2]);

				// Component specific chunks
				let cpu_chunk = Layout::default()
					.direction(Direction::Horizontal)
					.margin(0)
					.constraints(
						if app_state.left_legend {
							[Constraint::Percentage(15), Constraint::Percentage(85)]
						} else {
							[Constraint::Percentage(85), Constraint::Percentage(15)]
						}
						.as_ref(),
					)
					.split(vertical_chunks[0]);

				let network_chunk = Layout::default()
					.direction(Direction::Vertical)
					.margin(0)
					.constraints(
						if (bottom_chunks[0].height as f64 * 0.25) as u16 >= 4 {
							[Constraint::Percentage(75), Constraint::Percentage(25)]
						} else {
							let required = if bottom_chunks[0].height < 10 {
								bottom_chunks[0].height / 2
							} else {
								5
							};
							let remaining = bottom_chunks[0].height - required;
							[Constraint::Length(remaining), Constraint::Length(required)]
						}
						.as_ref(),
					)
					.split(bottom_chunks[0]);

				// Default chunk index based on left or right legend setting
				let legend_index = if app_state.left_legend { 0 } else { 1 };
				let graph_index = if app_state.left_legend { 1 } else { 0 };

				// Set up blocks and their components
				// CPU graph
				self.draw_cpu_graph(&mut f, &app_state, cpu_chunk[graph_index]);

				// CPU legend
				self.draw_cpu_legend(&mut f, app_state, cpu_chunk[legend_index]);

				//Memory usage graph
				self.draw_memory_graph(&mut f, &app_state, middle_chunks[0]);

				// Network graph
				self.draw_network_graph(&mut f, &app_state, network_chunk[0]);

				self.draw_network_labels(&mut f, app_state, network_chunk[1]);

				// Temperature table
				self.draw_temp_table(&mut f, app_state, middle_divided_chunk_2[0]);

				// Disk usage table
				self.draw_disk_table(&mut f, app_state, middle_divided_chunk_2[1]);

				// Processes table
				if app_state.is_searching() {
					let processes_chunk = Layout::default()
						.direction(Direction::Vertical)
						.margin(0)
						.constraints(
							if (bottom_chunks[1].height as f64 * 0.25) as u16 >= 4 {
								[Constraint::Percentage(75), Constraint::Percentage(25)]
							} else {
								let required = if bottom_chunks[1].height < 10 {
									bottom_chunks[1].height / 2
								} else {
									5
								};
								let remaining = bottom_chunks[1].height - required;
								[Constraint::Length(remaining), Constraint::Length(required)]
							}
							.as_ref(),
						)
						.split(bottom_chunks[1]);

					self.draw_processes_table(&mut f, app_state, processes_chunk[0]);
					self.draw_search_field(&mut f, app_state, processes_chunk[1]);
				} else {
					self.draw_processes_table(&mut f, app_state, bottom_chunks[1]);
				}
			}
		})?;

		Ok(())
	}

	fn draw_cpu_graph<B: backend::Backend>(
		&self, f: &mut Frame<B>, app_state: &app::App, draw_loc: Rect,
	) {
		let cpu_data: &[ConvertedCpuData] = &app_state.canvas_data.cpu_data;

		// CPU usage graph
		let x_axis: Axis<String> = Axis::default().bounds([0.0, TIME_STARTS_FROM as f64]);
		let y_axis = Axis::default()
			.style(self.colours.graph_style)
			.labels_style(self.colours.graph_style)
			.bounds([-0.5, 100.5])
			.labels(&["0%", "100%"]);

		let mut dataset_vector: Vec<Dataset> = Vec::new();
		let mut cpu_entries_vec: Vec<(Style, Vec<(f64, f64)>)> = Vec::new();

		for (i, cpu) in cpu_data.iter().enumerate() {
			cpu_entries_vec.push((
				self.colours.cpu_colour_styles[(i) % self.colours.cpu_colour_styles.len()],
				cpu.cpu_data
					.iter()
					.map(<(f64, f64)>::from)
					.collect::<Vec<_>>(),
			));
		}

		if app_state.show_average_cpu {
			if let Some(avg_cpu_entry) = cpu_data.first() {
				cpu_entries_vec.push((
					self.colours.cpu_colour_styles[0],
					avg_cpu_entry
						.cpu_data
						.iter()
						.map(<(f64, f64)>::from)
						.collect::<Vec<_>>(),
				));
			}
		}

		for cpu_entry in &cpu_entries_vec {
			dataset_vector.push(
				Dataset::default()
					.marker(if app_state.use_dot {
						Marker::Dot
					} else {
						Marker::Braille
					})
					.style(cpu_entry.0)
					.data(&(cpu_entry.1)),
			);
		}

		Chart::default()
			.block(
				Block::default()
					.title(" CPU ")
					.title_style(self.colours.widget_title_style)
					.borders(Borders::ALL)
					.border_style(match app_state.current_widget_selected {
						app::WidgetPosition::Cpu => self.colours.highlighted_border_style,
						_ => self.colours.border_style,
					}),
			)
			.x_axis(x_axis)
			.y_axis(y_axis)
			.datasets(&dataset_vector)
			.render(f, draw_loc);
	}

	fn draw_cpu_legend<B: backend::Backend>(
		&self, f: &mut Frame<B>, app_state: &mut app::App, draw_loc: Rect,
	) {
		let cpu_data: &[ConvertedCpuData] = &(app_state.canvas_data.cpu_data);

		let num_rows = max(0, i64::from(draw_loc.height) - 5) as u64;
		let start_position = get_start_position(
			num_rows,
			&(app_state.scroll_direction),
			&mut app_state.previous_cpu_table_position,
			app_state.currently_selected_cpu_table_position,
		);

		let sliced_cpu_data = &cpu_data[start_position as usize..];
		let mut stringified_cpu_data: Vec<Vec<String>> = Vec::new();

		for cpu in sliced_cpu_data {
			if let Some(cpu_data) = cpu.cpu_data.last() {
				stringified_cpu_data.push(vec![
					cpu.cpu_name.clone(),
					format!("{:.0}%", cpu_data.usage.round()),
				]);
			}
		}

		let mut cpu_row_counter: i64 = 0;

		let cpu_rows = stringified_cpu_data
			.iter()
			.enumerate()
			.map(|(itx, cpu_string_row)| {
				Row::StyledData(
					cpu_string_row.iter(),
					match app_state.current_widget_selected {
						app::WidgetPosition::Cpu => {
							if cpu_row_counter as u64
								== app_state.currently_selected_cpu_table_position - start_position
							{
								cpu_row_counter = -1;
								self.colours.currently_selected_text_style
							} else {
								if cpu_row_counter >= 0 {
									cpu_row_counter += 1;
								}
								self.colours.cpu_colour_styles
									[itx % self.colours.cpu_colour_styles.len()]
							}
						}
						_ => {
							self.colours.cpu_colour_styles
								[itx % self.colours.cpu_colour_styles.len()]
						}
					},
				)
			});

		// Calculate widths
		let width = f64::from(draw_loc.width);
		let width_ratios = vec![0.5, 0.5];
		let variable_intrinsic_results =
			get_variable_intrinsic_widths(width as u16, &width_ratios, &CPU_LEGEND_HEADER_LENS);
		let intrinsic_widths = &(variable_intrinsic_results.0)[0..variable_intrinsic_results.1];

		// Draw
		Table::new(CPU_LEGEND_HEADER.iter(), cpu_rows)
			.block(
				Block::default()
					.borders(Borders::ALL)
					.title_style(self.colours.widget_title_style)
					.border_style(match app_state.current_widget_selected {
						app::WidgetPosition::Cpu => self.colours.highlighted_border_style,
						_ => self.colours.border_style,
					}),
			)
			.header_style(self.colours.table_header_style)
			.widths(
				&(intrinsic_widths
					.into_iter()
					.map(|calculated_width| Constraint::Length(*calculated_width as u16))
					.collect::<Vec<_>>()),
			)
			.render(f, draw_loc);
	}

	fn draw_memory_graph<B: backend::Backend>(
		&self, f: &mut Frame<B>, app_state: &app::App, draw_loc: Rect,
	) {
		let mem_data: &[(f64, f64)] = &(app_state.canvas_data.mem_data);
		let swap_data: &[(f64, f64)] = &(app_state.canvas_data.swap_data);

		let x_axis: Axis<String> = Axis::default().bounds([0.0, TIME_STARTS_FROM as f64]);

		// Offset as the zero value isn't drawn otherwise...
		let y_axis: Axis<&str> = Axis::default()
			.style(self.colours.graph_style)
			.labels_style(self.colours.graph_style)
			.bounds([-0.5, 100.5])
			.labels(&["0%", "100%"]);

		let mut mem_canvas_vec: Vec<Dataset> = vec![Dataset::default()
			.name(&app_state.canvas_data.mem_label)
			.marker(if app_state.use_dot {
				Marker::Dot
			} else {
				Marker::Braille
			})
			.style(Style::default().fg(self.colours.ram_colour))
			.data(&mem_data)];

		if !(&swap_data).is_empty() {
			mem_canvas_vec.push(
				Dataset::default()
					.name(&app_state.canvas_data.swap_label)
					.marker(if app_state.use_dot {
						Marker::Dot
					} else {
						Marker::Braille
					})
					.style(Style::default().fg(self.colours.swap_colour))
					.data(&swap_data),
			);
		}

		Chart::default()
			.block(
				Block::default()
					.title(" Memory ")
					.title_style(self.colours.widget_title_style)
					.borders(Borders::ALL)
					.border_style(match app_state.current_widget_selected {
						app::WidgetPosition::Mem => self.colours.highlighted_border_style,
						_ => self.colours.border_style,
					}),
			)
			.x_axis(x_axis)
			.y_axis(y_axis)
			.datasets(&mem_canvas_vec)
			.render(f, draw_loc);
	}

	fn draw_network_graph<B: backend::Backend>(
		&self, f: &mut Frame<B>, app_state: &app::App, draw_loc: Rect,
	) {
		let network_data_rx: &[(f64, f64)] = &(app_state.canvas_data.network_data_rx);
		let network_data_tx: &[(f64, f64)] = &(app_state.canvas_data.network_data_tx);

		let x_axis: Axis<String> = Axis::default().bounds([0.0, 60_000.0]);
		let y_axis = Axis::default()
			.style(self.colours.graph_style)
			.labels_style(self.colours.graph_style)
			.bounds([-0.5, 30_f64])
			.labels(&["0B", "1KiB", "1MiB", "1GiB"]);
		Chart::default()
			.block(
				Block::default()
					.title(" Network ")
					.title_style(self.colours.widget_title_style)
					.borders(Borders::ALL)
					.border_style(match app_state.current_widget_selected {
						app::WidgetPosition::Network => self.colours.highlighted_border_style,
						_ => self.colours.border_style,
					}),
			)
			.x_axis(x_axis)
			.y_axis(y_axis)
			.datasets(&[
				Dataset::default()
					.name("RX")
					.marker(if app_state.use_dot {
						Marker::Dot
					} else {
						Marker::Braille
					})
					.style(Style::default().fg(self.colours.rx_colour))
					.data(&network_data_rx),
				Dataset::default()
					.name("TX")
					.marker(if app_state.use_dot {
						Marker::Dot
					} else {
						Marker::Braille
					})
					.style(Style::default().fg(self.colours.tx_colour))
					.data(&network_data_tx),
			])
			.render(f, draw_loc);
	}

	fn draw_network_labels<B: backend::Backend>(
		&self, f: &mut Frame<B>, app_state: &mut app::App, draw_loc: Rect,
	) {
		let rx_display: String = app_state.canvas_data.rx_display.clone();
		let tx_display: String = app_state.canvas_data.tx_display.clone();
		let total_rx_display: String = app_state.canvas_data.total_rx_display.clone();
		let total_tx_display: String = app_state.canvas_data.total_tx_display.clone();

		// Gross but I need it to work...
		let total_network = if cfg!(not(target_os = "windows")) {
			vec![vec![
				rx_display,
				tx_display,
				total_rx_display,
				total_tx_display,
			]]
		} else {
			vec![vec![rx_display, tx_display]]
		};
		let mapped_network = total_network
			.iter()
			.map(|val| Row::StyledData(val.iter(), self.colours.text_style));

		// Calculate widths
		let width_ratios: Vec<f64>;
		let lens: &Vec<usize>;
		let width = f64::from(draw_loc.width);

		if cfg!(not(target_os = "windows")) {
			width_ratios = vec![0.25, 0.25, 0.25, 0.25];
			lens = &NON_WINDOWS_NETWORK_HEADERS_LENS;
		} else {
			width_ratios = vec![0.25, 0.25];
			lens = &WINDOWS_NETWORK_HEADERS_LENS;
		}
		let variable_intrinsic_results =
			get_variable_intrinsic_widths(width as u16, &width_ratios, lens);
		let intrinsic_widths = &(variable_intrinsic_results.0)[0..variable_intrinsic_results.1];

		// Draw
		Table::new(
			if cfg!(not(target_os = "windows")) {
				NON_WINDOWS_NETWORK_HEADERS
			} else {
				WINDOWS_NETWORK_HEADERS
			}
			.iter(),
			mapped_network,
		)
		.block(
			Block::default()
				.borders(Borders::ALL)
				.title_style(self.colours.widget_title_style)
				.border_style(match app_state.current_widget_selected {
					app::WidgetPosition::Network => self.colours.highlighted_border_style,
					_ => self.colours.border_style,
				}),
		)
		.header_style(self.colours.table_header_style)
		.style(Style::default().fg(self.colours.text_colour))
		.widths(
			&(intrinsic_widths
				.into_iter()
				.map(|calculated_width| Constraint::Length(*calculated_width as u16))
				.collect::<Vec<_>>()),
		)
		.render(f, draw_loc);
	}

	fn draw_temp_table<B: backend::Backend>(
		&self, f: &mut Frame<B>, app_state: &mut app::App, draw_loc: Rect,
	) {
		let temp_sensor_data: &[Vec<String>] = &(app_state.canvas_data.temp_sensor_data);

		let num_rows = max(0, i64::from(draw_loc.height) - 5) as u64;
		let start_position = get_start_position(
			num_rows,
			&(app_state.scroll_direction),
			&mut app_state.previous_temp_position,
			app_state.currently_selected_temperature_position,
		);

		let sliced_vec = &(temp_sensor_data[start_position as usize..]);
		let mut temp_row_counter: i64 = 0;

		let temperature_rows = sliced_vec.iter().map(|temp_row| {
			Row::StyledData(
				temp_row.iter(),
				match app_state.current_widget_selected {
					app::WidgetPosition::Temp => {
						if temp_row_counter as u64
							== app_state.currently_selected_temperature_position - start_position
						{
							temp_row_counter = -1;
							self.colours.currently_selected_text_style
						} else {
							if temp_row_counter >= 0 {
								temp_row_counter += 1;
							}
							Style::default().fg(self.colours.text_colour)
						}
					}
					_ => Style::default().fg(self.colours.text_colour),
				},
			)
		});

		// Calculate widths
		let width = f64::from(draw_loc.width);
		let width_ratios = [0.5, 0.5];
		let variable_intrinsic_results =
			get_variable_intrinsic_widths(width as u16, &width_ratios, &TEMP_HEADERS_LENS);
		let intrinsic_widths = &(variable_intrinsic_results.0)[0..variable_intrinsic_results.1];

		// Draw
		Table::new(TEMP_HEADERS.iter(), temperature_rows)
			.block(
				Block::default()
					.title(" Temperatures ")
					.title_style(self.colours.widget_title_style)
					.borders(Borders::ALL)
					.border_style(match app_state.current_widget_selected {
						app::WidgetPosition::Temp => self.colours.highlighted_border_style,
						_ => self.colours.border_style,
					}),
			)
			.header_style(self.colours.table_header_style)
			.widths(
				&(intrinsic_widths
					.into_iter()
					.map(|calculated_width| Constraint::Length(*calculated_width as u16))
					.collect::<Vec<_>>()),
			)
			.render(f, draw_loc);
	}

	fn draw_disk_table<B: backend::Backend>(
		&self, f: &mut Frame<B>, app_state: &mut app::App, draw_loc: Rect,
	) {
		let disk_data: &[Vec<String>] = &(app_state.canvas_data.disk_data);
		let num_rows = max(0, i64::from(draw_loc.height) - 5) as u64;
		let start_position = get_start_position(
			num_rows,
			&(app_state.scroll_direction),
			&mut app_state.previous_disk_position,
			app_state.currently_selected_disk_position,
		);

		let sliced_vec = &disk_data[start_position as usize..];
		let mut disk_counter: i64 = 0;

		let disk_rows = sliced_vec.iter().map(|disk| {
			Row::StyledData(
				disk.iter(),
				match app_state.current_widget_selected {
					app::WidgetPosition::Disk => {
						if disk_counter as u64
							== app_state.currently_selected_disk_position - start_position
						{
							disk_counter = -1;
							self.colours.currently_selected_text_style
						} else {
							if disk_counter >= 0 {
								disk_counter += 1;
							}
							Style::default().fg(self.colours.text_colour)
						}
					}
					_ => Style::default().fg(self.colours.text_colour),
				},
			)
		});

		// Calculate widths
		// TODO: [PRETTY] Ellipsis on strings?
		let width = f64::from(draw_loc.width);
		let width_ratios = [0.2, 0.15, 0.13, 0.13, 0.13, 0.13, 0.13];
		let variable_intrinsic_results =
			get_variable_intrinsic_widths(width as u16, &width_ratios, &DISK_HEADERS_LENS);
		let intrinsic_widths = &variable_intrinsic_results.0[0..variable_intrinsic_results.1];

		// Draw!
		Table::new(DISK_HEADERS.iter(), disk_rows)
			.block(
				Block::default()
					.title(" Disk ")
					.title_style(self.colours.widget_title_style)
					.borders(Borders::ALL)
					.border_style(match app_state.current_widget_selected {
						app::WidgetPosition::Disk => self.colours.highlighted_border_style,
						_ => self.colours.border_style,
					}),
			)
			.header_style(self.colours.table_header_style)
			.widths(
				&(intrinsic_widths
					.into_iter()
					.map(|calculated_width| Constraint::Length(*calculated_width as u16))
					.collect::<Vec<_>>()),
			)
			.render(f, draw_loc);
	}

	fn draw_search_field<B: backend::Backend>(
		&self, f: &mut Frame<B>, app_state: &mut app::App, draw_loc: Rect,
	) {
		let width = max(0, draw_loc.width as i64 - 34) as u64;
		let query = app_state.get_current_search_query();
		let shrunk_query = if query.len() < width as usize {
			query
		} else {
			&query[(query.len() - width as usize)..]
		};

		let cursor_position = app_state.get_cursor_position();

		let query_with_cursor: Vec<Text> = if let app::WidgetPosition::ProcessSearch =
			app_state.current_widget_selected
		{
			if cursor_position >= query.len() {
				let mut q = vec![Text::styled(
					shrunk_query.to_string(),
					Style::default().fg(self.colours.text_colour),
				)];

				q.push(Text::styled(
					" ".to_string(),
					self.colours.currently_selected_text_style,
				));

				q
			} else {
				shrunk_query
					.chars()
					.enumerate()
					.map(|(itx, c)| {
						if let app::WidgetPosition::ProcessSearch =
							app_state.current_widget_selected
						{
							if itx == cursor_position {
								return Text::styled(
									c.to_string(),
									self.colours.currently_selected_text_style,
								);
							}
						}
						Text::styled(c.to_string(), Style::default().fg(self.colours.text_colour))
					})
					.collect::<Vec<_>>()
			}
		} else {
			vec![Text::styled(
				shrunk_query.to_string(),
				Style::default().fg(self.colours.text_colour),
			)]
		};

		let mut search_text = vec![if app_state.search_state.is_searching_with_pid() {
			Text::styled(
				"Search by PID (Tab for Name): ",
				self.colours.table_header_style,
			)
		} else {
			Text::styled(
				"Search by Name (Tab for PID): ",
				self.colours.table_header_style,
			)
		}];

		// Text options shamelessly stolen from VS Code.
		let option_text = vec![
			Text::styled("\n\n", self.colours.table_header_style),
			Text::styled("Match Case (Alt+C)", self.colours.table_header_style),
			if !app_state.search_state.is_ignoring_case() {
				Text::styled("[*]", self.colours.table_header_style)
			} else {
				Text::styled("[ ]", self.colours.table_header_style)
			},
			Text::styled("     ", self.colours.table_header_style),
			Text::styled("Match Whole Word (Alt+W)", self.colours.table_header_style),
			if app_state.search_state.is_searching_whole_word() {
				Text::styled("[*]", self.colours.table_header_style)
			} else {
				Text::styled("[ ]", self.colours.table_header_style)
			},
			Text::styled("     ", self.colours.table_header_style),
			Text::styled("Use Regex (Alt+R)", self.colours.table_header_style),
			if app_state.search_state.is_searching_with_regex() {
				Text::styled("[*]", self.colours.table_header_style)
			} else {
				Text::styled("[ ]", self.colours.table_header_style)
			},
		];

		search_text.extend(query_with_cursor);
		search_text.extend(option_text);

		Paragraph::new(search_text.iter())
			.block(
				Block::default()
					.borders(Borders::ALL)
					.title_style(self.colours.widget_title_style)
					.border_style(if app_state.get_current_regex_matcher().is_err() {
						Style::default().fg(Color::Red)
					} else {
						match app_state.current_widget_selected {
							app::WidgetPosition::ProcessSearch => {
								self.colours.highlighted_border_style
							}
							_ => self.colours.border_style,
						}
					}),
			)
			.style(Style::default().fg(self.colours.text_colour))
			.alignment(Alignment::Left)
			.wrap(false)
			.render(f, draw_loc);
	}

	fn draw_processes_table<B: backend::Backend>(
		&self, f: &mut Frame<B>, app_state: &mut app::App, draw_loc: Rect,
	) {
		let process_data: &[ConvertedProcessData] = &app_state.canvas_data.finalized_process_data;

		// Admittedly this is kinda a hack... but we need to:
		// * Scroll
		// * Show/hide elements based on scroll position
		//
		// As such, we use a process_counter to know when we've
		// hit the process we've currently scrolled to.
		// We also need to move the list - we can
		// do so by hiding some elements!
		let num_rows = max(0, i64::from(draw_loc.height) - 5) as u64;

		let position = get_start_position(
			num_rows,
			&(app_state.scroll_direction),
			&mut app_state.previous_process_position,
			app_state.currently_selected_process_position,
		);

		// Sanity check
		let start_position = if position >= process_data.len() as u64 {
			std::cmp::max(0, process_data.len() as i64 - 1) as u64
		} else {
			position
		};

		let sliced_vec = &(process_data[start_position as usize..]);
		let mut process_counter: i64 = 0;

		// Draw!
		let process_rows = sliced_vec.iter().map(|process| {
			let stringified_process_vec: Vec<String> = vec![
				if app_state.is_grouped() {
					process.group_pids.len().to_string()
				} else {
					process.pid.to_string()
				},
				process.name.clone(),
				format!("{:.1}%", process.cpu_usage),
				format!("{:.1}%", process.mem_usage),
			];
			Row::StyledData(
				stringified_process_vec.into_iter(),
				match app_state.current_widget_selected {
					app::WidgetPosition::Process => {
						if process_counter as u64
							== app_state.currently_selected_process_position - start_position
						{
							process_counter = -1;
							self.colours.currently_selected_text_style
						} else {
							if process_counter >= 0 {
								process_counter += 1;
							}
							Style::default().fg(self.colours.text_colour)
						}
					}
					_ => Style::default().fg(self.colours.text_colour),
				},
			)
		});

		use app::data_harvester::processes::ProcessSorting;
		let mut pid_or_name = if app_state.is_grouped() {
			"Count"
		} else {
			"PID(p)"
		}
		.to_string();
		let mut name = "Name(n)".to_string();
		let mut cpu = "CPU%(c)".to_string();
		let mut mem = "Mem%(m)".to_string();

		let direction_val = if app_state.process_sorting_reverse {
			"⯆".to_string()
		} else {
			"⯅".to_string()
		};

		match app_state.process_sorting_type {
			ProcessSorting::CPU => cpu += &direction_val,
			ProcessSorting::MEM => mem += &direction_val,
			ProcessSorting::PID => pid_or_name += &direction_val,
			ProcessSorting::NAME => name += &direction_val,
		};

		let process_headers = [pid_or_name, name, cpu, mem];
		let process_headers_lens: Vec<usize> = process_headers
			.iter()
			.map(|entry| entry.len())
			.collect::<Vec<_>>();

		// Calculate widths
		let width = f64::from(draw_loc.width);
		let width_ratios = [0.2, 0.4, 0.2, 0.2];
		let variable_intrinsic_results =
			get_variable_intrinsic_widths(width as u16, &width_ratios, &process_headers_lens);
		let intrinsic_widths = &(variable_intrinsic_results.0)[0..variable_intrinsic_results.1];

		Table::new(process_headers.iter(), process_rows)
			.block(
				Block::default()
					.title(" Processes ")
					.title_style(self.colours.widget_title_style)
					.borders(Borders::ALL)
					.border_style(match app_state.current_widget_selected {
						app::WidgetPosition::Process => self.colours.highlighted_border_style,
						_ => self.colours.border_style,
					}),
			)
			.header_style(self.colours.table_header_style)
			.widths(
				&(intrinsic_widths
					.into_iter()
					.map(|calculated_width| Constraint::Length(*calculated_width as u16))
					.collect::<Vec<_>>()),
			)
			.render(f, draw_loc);
	}
}

/// A somewhat jury-rigged solution to simulate a variable intrinsic layout for
/// table widths.  Note that this will do one main pass to try to properly
/// allocate widths.  This will thus potentially cut off latter elements
/// (return size of 0) if it is too small (threshold), but will try its best.
///
/// `width thresholds` and `desired_widths_ratio` should be the same length.
/// Otherwise bad things happen.
fn get_variable_intrinsic_widths(
	total_width: u16, desired_widths_ratio: &[f64], width_thresholds: &[usize],
) -> (Vec<u16>, usize) {
	let num_widths = desired_widths_ratio.len();
	let mut resulting_widths: Vec<u16> = vec![0; num_widths];
	let mut last_index = 0;

	let mut remaining_width = (total_width - (num_widths as u16 - 1)) as i32; // Required for spaces...
	let desired_widths = desired_widths_ratio
		.iter()
		.map(|&desired_width_ratio| (desired_width_ratio * total_width as f64) as i32)
		.collect::<Vec<_>>();

	for (itx, desired_width) in desired_widths.into_iter().enumerate() {
		resulting_widths[itx] = if desired_width < width_thresholds[itx] as i32 {
			// Try to take threshold, else, 0
			if remaining_width < width_thresholds[itx] as i32 {
				0
			} else {
				remaining_width -= width_thresholds[itx] as i32;
				width_thresholds[itx] as u16
			}
		} else {
			// Take as large as possible
			if remaining_width < desired_width {
				// Check the biggest chunk possible
				if remaining_width < width_thresholds[itx] as i32 {
					0
				} else {
					let temp_width = remaining_width;
					remaining_width = 0;
					temp_width as u16
				}
			} else {
				remaining_width -= desired_width;
				desired_width as u16
			}
		};

		if resulting_widths[itx] == 0 {
			break;
		} else {
			last_index += 1;
		}
	}

	// Simple redistribution tactic - if there's any space left, split it evenly amongst all members
	if last_index < num_widths {
		let for_all_widths = (remaining_width / last_index as i32) as u16;
		let mut remainder = remaining_width % last_index as i32;

		for resulting_width in &mut resulting_widths {
			*resulting_width += for_all_widths;
			if remainder > 0 {
				*resulting_width += 1;
				remainder -= 1;
			}
		}
	}

	(resulting_widths, last_index)
}

fn get_start_position(
	num_rows: u64, scroll_direction: &app::ScrollDirection, previously_scrolled_position: &mut u64,
	currently_selected_position: u64,
) -> u64 {
	match scroll_direction {
		app::ScrollDirection::DOWN => {
			if currently_selected_position < *previously_scrolled_position + num_rows {
				// If, using previous_scrolled_position, we can see the element
				// (so within that and + num_rows) just reuse the current previously scrolled position
				*previously_scrolled_position
			} else if currently_selected_position >= num_rows {
				// Else if the current position past the last element visible in the list, omit
				// until we can see that element
				*previously_scrolled_position = currently_selected_position - num_rows;
				currently_selected_position - num_rows
			} else {
				// Else, if it is not past the last element visible, do not omit anything
				0
			}
		}
		app::ScrollDirection::UP => {
			if currently_selected_position <= *previously_scrolled_position {
				// If it's past the first element, then show from that element downwards
				*previously_scrolled_position = currently_selected_position;
				currently_selected_position
			} else {
				// Else, don't change what our start position is from whatever it is set to!
				*previously_scrolled_position
			}
		}
	}
}
