use crate::{app, constants, utils::error, utils::gen_util::*};
use tui::{
	backend,
	layout::{Alignment, Constraint, Direction, Layout, Rect},
	style::{Color, Modifier, Style},
	terminal::Frame,
	widgets::{Axis, Block, Borders, Chart, Dataset, Marker, Paragraph, Row, Table, Text, Widget},
	Terminal,
};

const TEXT_COLOUR: Color = Color::Gray;
const GRAPH_COLOUR: Color = Color::Gray;
const BORDER_STYLE_COLOUR: Color = Color::Gray;
const HIGHLIGHTED_BORDER_STYLE_COLOUR: Color = Color::LightBlue;
const GOLDEN_RATIO: f32 = 0.618_034; // Approx, good enough for use (also Clippy gets mad if it's too long)

lazy_static! {
	static ref HELP_TEXT: [Text<'static>; 14] = [
		Text::raw("\nGeneral Keybindings\n"),
		Text::raw("q, Ctrl-c to quit.\n"),
		Text::raw("Ctrl-r to reset all data.\n"),
		Text::raw("f to toggle freezing and unfreezing the display.\n"),
		Text::raw("Ctrl+Up/k, Ctrl+Down/j, Ctrl+Left/h, Ctrl+Right/l to navigate between panels.\n"),
		Text::raw("Up and Down scrolls through a list.\n"),
		Text::raw("Esc to close a dialog window (help or dd confirmation).\n"),
		Text::raw("? to get this help screen.\n"),
		Text::raw("\n Process Panel Keybindings\n"),
		Text::raw("dd to kill the selected process.\n"),
		Text::raw("c to sort by CPU usage.\n"),
		Text::raw("m to sort by memory usage.\n"),
		Text::raw("p to sort by PID.\n"),
		Text::raw("n to sort by process name.\n"),
	];
	static ref COLOUR_LIST: Vec<Color> = gen_n_colours(constants::NUM_COLOURS);
	static ref CANVAS_BORDER_STYLE: Style = Style::default().fg(BORDER_STYLE_COLOUR);
	static ref CANVAS_HIGHLIGHTED_BORDER_STYLE: Style = Style::default().fg(HIGHLIGHTED_BORDER_STYLE_COLOUR);
}

#[derive(Default)]
pub struct CanvasData {
	pub rx_display: String,
	pub tx_display: String,
	pub total_rx_display: String,
	pub total_tx_display: String,
	pub network_data_rx: Vec<(f64, f64)>,
	pub network_data_tx: Vec<(f64, f64)>,
	pub disk_data: Vec<Vec<String>>,
	pub temp_sensor_data: Vec<Vec<String>>,
	pub process_data: Vec<Vec<String>>,
	pub memory_labels: Vec<(u64, u64)>,
	pub mem_data: Vec<(f64, f64)>,
	pub swap_data: Vec<(f64, f64)>,
	pub cpu_data: Vec<(String, Vec<(f64, f64)>)>,
}

/// Generates random colours.
/// Strategy found from https://martin.ankerl.com/2009/12/09/how-to-create-random-colors-programmatically/
fn gen_n_colours(num_to_gen: i32) -> Vec<Color> {
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
	let mut colour_vec: Vec<Color> = vec![
		Color::LightBlue,
		Color::LightYellow,
		Color::Red,
		Color::Green,
		Color::LightCyan,
		Color::LightMagenta,
	];

	let mut h: f32 = 0.4; // We don't need random colours... right?
	for _i in 0..num_to_gen {
		h = gen_hsv(h);
		let result = hsv_to_rgb(h, 0.5, 0.95);
		colour_vec.push(Color::Rgb(result.0, result.1, result.2));
	}

	colour_vec
}

pub fn draw_data<B: backend::Backend>(terminal: &mut Terminal<B>, app_state: &mut app::App, canvas_data: &CanvasData) -> error::Result<()> {
	terminal.autoresize()?;
	terminal.draw(|mut f| {
		if app_state.show_help {
			// Only for the dialog (help, dd) menus
			let vertical_dialog_chunk = Layout::default()
				.direction(Direction::Vertical)
				.margin(1)
				.constraints([Constraint::Percentage(32), Constraint::Percentage(40), Constraint::Percentage(28)].as_ref())
				.split(f.size());

			let middle_dialog_chunk = Layout::default()
				.direction(Direction::Horizontal)
				.margin(0)
				.constraints([Constraint::Percentage(30), Constraint::Percentage(40), Constraint::Percentage(30)].as_ref())
				.split(vertical_dialog_chunk[1]);

			Paragraph::new(HELP_TEXT.iter())
				.block(Block::default().title("Help (Press Esc to close)").borders(Borders::ALL))
				.style(Style::default().fg(Color::Gray))
				.alignment(Alignment::Left)
				.wrap(true)
				.render(&mut f, middle_dialog_chunk[1]);
		} else {
			let vertical_chunks = Layout::default()
				.direction(Direction::Vertical)
				.margin(1)
				.constraints([Constraint::Percentage(33), Constraint::Percentage(34), Constraint::Percentage(34)].as_ref())
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
						[Constraint::Percentage(10), Constraint::Percentage(90)]
					} else {
						[Constraint::Percentage(90), Constraint::Percentage(10)]
					}
					.as_ref(),
				)
				.split(vertical_chunks[0]);

			let mem_chunk = Layout::default()
				.direction(Direction::Horizontal)
				.margin(0)
				.constraints(
					if app_state.left_legend {
						[Constraint::Percentage(10), Constraint::Percentage(90)]
					} else {
						[Constraint::Percentage(90), Constraint::Percentage(10)]
					}
					.as_ref(),
				)
				.split(middle_chunks[0]);

			let network_chunk = Layout::default()
				.direction(Direction::Horizontal)
				.margin(0)
				.constraints(
					if app_state.left_legend {
						[Constraint::Percentage(10), Constraint::Percentage(90)]
					} else {
						[Constraint::Percentage(90), Constraint::Percentage(10)]
					}
					.as_ref(),
				)
				.split(bottom_chunks[0]);

			// Default chunk index based on left or right legend setting
			let legend_index = if app_state.left_legend { 0 } else { 1 };
			let graph_index = if app_state.left_legend { 1 } else { 0 };

			// Set up blocks and their components
			// CPU graph
			draw_cpu_graph(&mut f, &app_state, &canvas_data.cpu_data, cpu_chunk[graph_index]);

			// CPU label

			//Memory usage graph
			draw_memory_graph(&mut f, &app_state, &canvas_data.mem_data, &canvas_data.swap_data, mem_chunk[graph_index]);

			// Memory label

			// Temperature table
			draw_temp_table(&mut f, app_state, &canvas_data.temp_sensor_data, middle_divided_chunk_2[0]);

			// Disk usage table
			draw_disk_table(&mut f, app_state, &canvas_data.disk_data, middle_divided_chunk_2[1]);

			// Network graph
			draw_network_graph(
				&mut f,
				&app_state,
				&canvas_data.network_data_rx,
				&canvas_data.network_data_tx,
				network_chunk[graph_index],
			);

			// Network label

			// Processes table
			draw_processes_table(&mut f, app_state, &canvas_data.process_data, bottom_chunks[1]);
		}
	})?;

	Ok(())
}

fn draw_cpu_graph<B: backend::Backend>(f: &mut Frame<B>, app_state: &app::App, cpu_data: &[(String, Vec<(f64, f64)>)], draw_loc: Rect) {
	// CPU usage graph

	let x_axis: Axis<String> = Axis::default()
		.style(Style::default().fg(GRAPH_COLOUR))
		.bounds([0.0, constants::TIME_STARTS_FROM as f64 * 10.0]);
	let y_axis = Axis::default()
		.style(Style::default().fg(GRAPH_COLOUR))
		.bounds([-0.5, 100.5])
		.labels(&["0%", "100%"]);

	let mut dataset_vector: Vec<Dataset> = Vec::new();

	for (i, cpu) in cpu_data.iter().enumerate() {
		let mut avg_cpu_exist_offset = 0;
		if app_state.show_average_cpu {
			if i == 0 {
				// Skip, we want to render the average cpu last!
				continue;
			} else {
				avg_cpu_exist_offset = 1;
			}
		}

		dataset_vector.push(
			Dataset::default()
				.name(&cpu.0)
				.marker(if app_state.use_dot { Marker::Dot } else { Marker::Braille })
				.style(Style::default().fg(COLOUR_LIST[(i - avg_cpu_exist_offset) % COLOUR_LIST.len()]))
				.data(&(cpu.1)),
		);
	}

	if !cpu_data.is_empty() && app_state.show_average_cpu {
		// Unwrap should be safe here, this assumes that the cpu_data vector is populated...
		dataset_vector.push(
			Dataset::default()
				//.name(&cpu_data.first().unwrap().0)
				.marker(if app_state.use_dot { Marker::Dot } else { Marker::Braille })
				.style(Style::default().fg(COLOUR_LIST[(cpu_data.len() - 1) % COLOUR_LIST.len()]))
				.data(&(cpu_data.first().unwrap().1)),
		);
	}

	Chart::default()
		.block(
			Block::default()
				.title("CPU Usage")
				.borders(Borders::ALL)
				.border_style(match app_state.current_application_position {
					app::ApplicationPosition::CPU => *CANVAS_HIGHLIGHTED_BORDER_STYLE,
					_ => *CANVAS_BORDER_STYLE,
				}),
		)
		.x_axis(x_axis)
		.y_axis(y_axis)
		.datasets(&dataset_vector)
		.render(f, draw_loc);
}

fn draw_cpu_legend() {}

fn draw_memory_graph<B: backend::Backend>(f: &mut Frame<B>, app_state: &app::App, swap_data: &[(f64, f64)], mem_data: &[(f64, f64)], draw_loc: Rect) {
	let x_axis: Axis<String> = Axis::default()
		.style(Style::default().fg(GRAPH_COLOUR))
		.bounds([0.0, constants::TIME_STARTS_FROM as f64 * 10.0]);
	let y_axis = Axis::default()
		.style(Style::default().fg(GRAPH_COLOUR))
		.bounds([-0.5, 100.5]) // Offset as the zero value isn't drawn otherwise...
		.labels(&["0%", "100%"]);

	let mut mem_canvas_vec: Vec<Dataset> = vec![Dataset::default()
		//.name(&mem_name)
		.marker(if app_state.use_dot { Marker::Dot } else { Marker::Braille })
		.style(Style::default().fg(COLOUR_LIST[0]))
		.data(&mem_data)];

	if !(&swap_data).is_empty() {
		if let Some(last_canvas_result) = (&swap_data).last() {
			if last_canvas_result.1 >= 0.0 {
				mem_canvas_vec.push(
					Dataset::default()
						//.name(&swap_name)
						.marker(if app_state.use_dot { Marker::Dot } else { Marker::Braille })
						.style(Style::default().fg(COLOUR_LIST[1]))
						.data(&swap_data),
				);
			}
		}
	}

	Chart::default()
		.block(
			Block::default()
				.title("Memory Usage")
				.borders(Borders::ALL)
				.border_style(match app_state.current_application_position {
					app::ApplicationPosition::MEM => *CANVAS_HIGHLIGHTED_BORDER_STYLE,
					_ => *CANVAS_BORDER_STYLE,
				}),
		)
		.x_axis(x_axis)
		.y_axis(y_axis)
		.datasets(&mem_canvas_vec)
		.render(f, draw_loc);
}

fn draw_memory_legend() {}

fn draw_network_graph<B: backend::Backend>(
	f: &mut Frame<B>, app_state: &app::App, network_data_rx: &[(f64, f64)], network_data_tx: &[(f64, f64)], draw_loc: Rect,
) {
	let x_axis: Axis<String> = Axis::default().style(Style::default().fg(GRAPH_COLOUR)).bounds([0.0, 600_000.0]);
	let y_axis = Axis::default()
		.style(Style::default().fg(GRAPH_COLOUR))
		.bounds([-0.5, 30_f64])
		.labels(&["0B", "1KiB", "1MiB", "1GiB"]);
	Chart::default()
		.block(
			Block::default()
				.title("Network")
				.borders(Borders::ALL)
				.border_style(match app_state.current_application_position {
					app::ApplicationPosition::NETWORK => *CANVAS_HIGHLIGHTED_BORDER_STYLE,
					_ => *CANVAS_BORDER_STYLE,
				}),
		)
		.x_axis(x_axis)
		.y_axis(y_axis)
		.datasets(&[
			Dataset::default()
				.marker(if app_state.use_dot { Marker::Dot } else { Marker::Braille })
				.style(Style::default().fg(COLOUR_LIST[0]))
				.data(&network_data_rx),
			Dataset::default()
				.marker(if app_state.use_dot { Marker::Dot } else { Marker::Braille })
				.style(Style::default().fg(COLOUR_LIST[1]))
				.data(&network_data_tx),
		])
		.render(f, draw_loc);
}

fn draw_network_legend() {}

fn draw_temp_table<B: backend::Backend>(f: &mut Frame<B>, app_state: &mut app::App, temp_sensor_data: &[Vec<String>], draw_loc: Rect) {
	let num_rows = i64::from(draw_loc.height) - 4;
	let start_position = get_start_position(
		num_rows,
		&(app_state.scroll_direction),
		&mut app_state.previous_temp_position,
		&mut app_state.currently_selected_temperature_position,
	);

	let sliced_vec: Vec<Vec<String>> = (&temp_sensor_data[start_position as usize..]).to_vec();
	let mut disk_counter = 0;

	let temperature_rows = sliced_vec.iter().map(|disk| {
		Row::StyledData(
			disk.iter(),
			if disk_counter == app_state.currently_selected_temperature_position - start_position {
				disk_counter = -1;
				Style::default().fg(Color::Black).bg(Color::Cyan)
			} else {
				if disk_counter >= 0 {
					disk_counter += 1;
				}
				Style::default().fg(TEXT_COLOUR)
			},
		)
	});

	let width = f64::from(draw_loc.width);
	Table::new(["Sensor", "Temp"].iter(), temperature_rows)
		.block(
			Block::default()
				.title("Temperatures")
				.borders(Borders::ALL)
				.border_style(match app_state.current_application_position {
					app::ApplicationPosition::TEMP => *CANVAS_HIGHLIGHTED_BORDER_STYLE,
					_ => *CANVAS_BORDER_STYLE,
				}),
		)
		.header_style(Style::default().fg(Color::LightBlue))
		.widths(&[Constraint::Length((width * 0.45) as u16), Constraint::Length((width * 0.4) as u16)])
		.render(f, draw_loc);
}

fn draw_disk_table<B: backend::Backend>(f: &mut Frame<B>, app_state: &mut app::App, disk_data: &[Vec<String>], draw_loc: Rect) {
	let num_rows = i64::from(draw_loc.height) - 4;
	let start_position = get_start_position(
		num_rows,
		&(app_state.scroll_direction),
		&mut app_state.previous_disk_position,
		&mut app_state.currently_selected_disk_position,
	);

	let sliced_vec: Vec<Vec<String>> = (&disk_data[start_position as usize..]).to_vec();
	let mut disk_counter = 0;

	let disk_rows = sliced_vec.iter().map(|disk| {
		Row::StyledData(
			disk.iter(),
			if disk_counter == app_state.currently_selected_disk_position - start_position {
				disk_counter = -1;
				Style::default().fg(Color::Black).bg(Color::Cyan)
			} else {
				if disk_counter >= 0 {
					disk_counter += 1;
				}
				Style::default().fg(TEXT_COLOUR)
			},
		)
	});

	// TODO: We may have to dynamically remove some of these table elements based on size...
	let width = f64::from(draw_loc.width);
	Table::new(["Disk", "Mount", "Used", "Total", "Free", "R/s", "W/s"].iter(), disk_rows)
		.block(
			Block::default()
				.title("Disk Usage")
				.borders(Borders::ALL)
				.border_style(match app_state.current_application_position {
					app::ApplicationPosition::DISK => *CANVAS_HIGHLIGHTED_BORDER_STYLE,
					_ => *CANVAS_BORDER_STYLE,
				}),
		)
		.header_style(Style::default().fg(Color::LightBlue).modifier(Modifier::BOLD))
		.widths(&[
			Constraint::Length((width * 0.18).floor() as u16),
			Constraint::Length((width * 0.14).floor() as u16),
			Constraint::Length((width * 0.11).floor() as u16),
			Constraint::Length((width * 0.11).floor() as u16),
			Constraint::Length((width * 0.11).floor() as u16),
			Constraint::Length((width * 0.11).floor() as u16),
			Constraint::Length((width * 0.11).floor() as u16),
		])
		.render(f, draw_loc);
}

fn draw_processes_table<B: backend::Backend>(f: &mut Frame<B>, app_state: &mut app::App, process_data: &[Vec<String>], draw_loc: Rect) {
	let width = f64::from(draw_loc.width);

	// Admittedly this is kinda a hack... but we need to:
	// * Scroll
	// * Show/hide elements based on scroll position
	// As such, we use a process_counter to know when we've hit the process we've currently scrolled to.  We also need to move the list - we can
	// do so by hiding some elements!
	let num_rows = i64::from(draw_loc.height) - 4;

	let start_position = get_start_position(
		num_rows,
		&(app_state.scroll_direction),
		&mut app_state.previous_process_position,
		&mut app_state.currently_selected_process_position,
	);

	let sliced_vec: Vec<Vec<String>> = (&process_data[start_position as usize..]).to_vec();
	let mut process_counter = 0;

	let process_rows = sliced_vec.iter().map(|process| {
		Row::StyledData(
			process.iter(),
			if process_counter == app_state.currently_selected_process_position - start_position {
				process_counter = -1;
				Style::default().fg(Color::Black).bg(Color::Cyan)
			} else {
				if process_counter >= 0 {
					process_counter += 1;
				}
				Style::default().fg(TEXT_COLOUR)
			},
		)
	});

	{
		use app::data_collection::processes::ProcessSorting;
		let mut pid = "PID(p)".to_string();
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
			ProcessSorting::PID => pid += &direction_val,
			ProcessSorting::NAME => name += &direction_val,
		};

		Table::new([pid, name, cpu, mem].iter(), process_rows)
			.block(
				Block::default()
					.title("Processes")
					.borders(Borders::ALL)
					.border_style(match app_state.current_application_position {
						app::ApplicationPosition::PROCESS => *CANVAS_HIGHLIGHTED_BORDER_STYLE,
						_ => *CANVAS_BORDER_STYLE,
					}),
			)
			.header_style(Style::default().fg(Color::LightBlue))
			.widths(&[
				Constraint::Length((width * 0.2) as u16),
				Constraint::Length((width * 0.35) as u16),
				Constraint::Length((width * 0.2) as u16),
				Constraint::Length((width * 0.2) as u16),
			])
			.render(f, draw_loc);
	}
}

fn get_start_position(
	num_rows: i64, scroll_direction: &app::ScrollDirection, previous_position: &mut i64, currently_selected_position: &mut i64,
) -> i64 {
	match scroll_direction {
		app::ScrollDirection::DOWN => {
			if *currently_selected_position < num_rows {
				0
			} else if *currently_selected_position - num_rows < *previous_position {
				*previous_position
			} else {
				*previous_position = *currently_selected_position - num_rows + 1;
				*previous_position
			}
		}
		app::ScrollDirection::UP => {
			if *currently_selected_position == *previous_position - 1 {
				*previous_position = if *previous_position > 0 { *previous_position - 1 } else { 0 };
				*previous_position
			} else {
				*previous_position
			}
		}
	}
}
