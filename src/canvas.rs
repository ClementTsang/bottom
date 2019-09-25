use tui_temp_fork::{
	backend,
	layout::{Constraint, Direction, Layout},
	style::{Color, Modifier, Style},
	widgets::{Axis, Block, Borders, Chart, Dataset, Marker, Row, Table, Widget},
	Terminal,
};

use crate::{app, utils::error};

const COLOUR_LIST : [Color; 6] = [Color::Red, Color::Green, Color::LightYellow, Color::LightBlue, Color::LightCyan, Color::LightMagenta];
const TEXT_COLOUR : Color = Color::Gray;
const GRAPH_COLOUR : Color = Color::Gray;
const BORDER_STYLE_COLOUR : Color = Color::Gray;
const HIGHLIGHTED_BORDER_STYLE_COLOUR : Color = Color::LightBlue;

#[derive(Default)]
pub struct CanvasData {
	pub rx_display : String,
	pub tx_display : String,
	pub network_data_rx : Vec<(f64, f64)>,
	pub network_data_tx : Vec<(f64, f64)>,
	pub disk_data : Vec<Vec<String>>,
	pub temp_sensor_data : Vec<Vec<String>>,
	pub process_data : Vec<Vec<String>>,
	pub mem_data : Vec<(f64, f64)>,
	pub swap_data : Vec<(f64, f64)>,
	pub cpu_data : Vec<(String, Vec<(f64, f64)>)>,
}

pub fn draw_data<B : backend::Backend>(terminal : &mut Terminal<B>, app_state : &mut app::App, canvas_data : &CanvasData) -> error::Result<()> {
	let border_style : Style = Style::default().fg(BORDER_STYLE_COLOUR);
	let highlighted_border_style : Style = Style::default().fg(HIGHLIGHTED_BORDER_STYLE_COLOUR);

	let temperature_rows = canvas_data
		.temp_sensor_data
		.iter()
		.map(|sensor| Row::StyledData(sensor.iter(), Style::default().fg(TEXT_COLOUR)));
	let disk_rows = canvas_data.disk_data.iter().map(|disk| Row::StyledData(disk.iter(), Style::default().fg(TEXT_COLOUR)));

	terminal.draw(|mut f| {
		//debug!("Drawing!");
		let vertical_chunks = Layout::default()
			.direction(Direction::Vertical)
			.margin(1)
			.constraints([Constraint::Percentage(34), Constraint::Percentage(34), Constraint::Percentage(33)].as_ref())
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

		// Set up blocks and their components
		// CPU usage graph
		{
			let x_axis : Axis<String> = Axis::default().style(Style::default().fg(GRAPH_COLOUR)).bounds([0.0, 600_000.0]);
			let y_axis = Axis::default().style(Style::default().fg(GRAPH_COLOUR)).bounds([-0.5, 100.5]).labels(&["0%", "100%"]);

			let mut dataset_vector : Vec<Dataset> = Vec::new();

			for (i, cpu) in canvas_data.cpu_data.iter().enumerate() {
				let mut avg_cpu_exist_offset = 0;
				if app_state.show_average_cpu {
					if i == 0 {
						// Skip, we want to render the average cpu last!
						continue;
					}
					else {
						avg_cpu_exist_offset = 1;
					}
				}

				dataset_vector.push(
					Dataset::default()
						.name(&cpu.0)
						.marker(if app_state.use_dot { Marker::Dot } else { Marker::Braille })
						.style(Style::default().fg(COLOUR_LIST[i - avg_cpu_exist_offset % COLOUR_LIST.len()]))
						.data(&(cpu.1)),
				);
			}

			if !canvas_data.cpu_data.is_empty() && app_state.show_average_cpu {
				dataset_vector.push(
					Dataset::default()
						.name(&canvas_data.cpu_data[0].0)
						.marker(if app_state.use_dot { Marker::Dot } else { Marker::Braille })
						.style(Style::default().fg(COLOUR_LIST[canvas_data.cpu_data.len() - 1 % COLOUR_LIST.len()]))
						.data(&(canvas_data.cpu_data[0].1)),
				);
			}

			Chart::default()
				.block(
					Block::default()
						.title("CPU Usage")
						.borders(Borders::ALL)
						.border_style(match app_state.current_application_position {
							app::ApplicationPosition::CPU => highlighted_border_style,
							_ => border_style,
						}),
				)
				.x_axis(x_axis)
				.y_axis(y_axis)
				.datasets(&dataset_vector)
				.render(&mut f, vertical_chunks[0]);
		}

		//Memory usage graph
		{
			let x_axis : Axis<String> = Axis::default().style(Style::default().fg(GRAPH_COLOUR)).bounds([0.0, 600_000.0]);
			let y_axis = Axis::default().style(Style::default().fg(GRAPH_COLOUR)).bounds([-0.5, 100.5]).labels(&["0%", "100%"]); // Offset as the zero value isn't drawn otherwise...
			let mem_name = "RAM:".to_string() + &format!("{:3}%", (canvas_data.mem_data.last().unwrap_or(&(0_f64, 0_f64)).1.round() as u64));
			let swap_name = "SWP:".to_string() + &format!("{:3}%", (canvas_data.swap_data.last().unwrap_or(&(0_f64, 0_f64)).1.round() as u64));

			let mut mem_canvas_vec : Vec<Dataset> = vec![Dataset::default()
				.name(&mem_name)
				.marker(if app_state.use_dot { Marker::Dot } else { Marker::Braille })
				.style(Style::default().fg(Color::LightBlue))
				.data(&canvas_data.mem_data)];

			if !(&canvas_data.swap_data).is_empty() && (&canvas_data.swap_data).last().unwrap().1 >= 0.0 {
				mem_canvas_vec.push(
					Dataset::default()
						.name(&swap_name)
						.marker(if app_state.use_dot { Marker::Dot } else { Marker::Braille })
						.style(Style::default().fg(Color::LightYellow))
						.data(&canvas_data.swap_data),
				);
			}

			Chart::default()
				.block(
					Block::default()
						.title("Memory Usage")
						.borders(Borders::ALL)
						.border_style(match app_state.current_application_position {
							app::ApplicationPosition::MEM => highlighted_border_style,
							_ => border_style,
						}),
				)
				.x_axis(x_axis)
				.y_axis(y_axis)
				.datasets(&mem_canvas_vec)
				.render(&mut f, middle_chunks[0]);
		}

		// Temperature table
		{
			let width = f64::from(middle_divided_chunk_2[0].width);
			Table::new(["Sensor", "Temp"].iter(), temperature_rows)
				.block(
					Block::default()
						.title("Temperatures")
						.borders(Borders::ALL)
						.border_style(match app_state.current_application_position {
							app::ApplicationPosition::TEMP => highlighted_border_style,
							_ => border_style,
						}),
				)
				.header_style(Style::default().fg(Color::LightBlue))
				.widths(&[(width * 0.45) as u16, (width * 0.4) as u16])
				.render(&mut f, middle_divided_chunk_2[0]);
		}

		// Disk usage table
		{
			// TODO: We may have to dynamically remove some of these table elements based on size...
			let width = f64::from(middle_divided_chunk_2[1].width);
			Table::new(["Disk", "Mount", "Used", "Total", "Free", "R/s", "W/s"].iter(), disk_rows)
				.block(
					Block::default()
						.title("Disk Usage")
						.borders(Borders::ALL)
						.border_style(match app_state.current_application_position {
							app::ApplicationPosition::DISK => highlighted_border_style,
							_ => border_style,
						}),
				)
				.header_style(Style::default().fg(Color::LightBlue).modifier(Modifier::BOLD))
				.widths(&[
					(width * 0.18).floor() as u16,
					(width * 0.14).floor() as u16,
					(width * 0.11).floor() as u16,
					(width * 0.11).floor() as u16,
					(width * 0.11).floor() as u16,
					(width * 0.11).floor() as u16,
					(width * 0.11).floor() as u16,
				])
				.render(&mut f, middle_divided_chunk_2[1]);
		}

		// Network graph
		{
			let x_axis : Axis<String> = Axis::default().style(Style::default().fg(GRAPH_COLOUR)).bounds([0.0, 600_000.0]);
			let y_axis = Axis::default().style(Style::default().fg(GRAPH_COLOUR)).bounds([-0.5, 1_000_000.5]).labels(&["0GB", "1GB"]);
			Chart::default()
				.block(
					Block::default()
						.title("Network")
						.borders(Borders::ALL)
						.border_style(match app_state.current_application_position {
							app::ApplicationPosition::NETWORK => highlighted_border_style,
							_ => border_style,
						}),
				)
				.x_axis(x_axis)
				.y_axis(y_axis)
				.datasets(&[
					Dataset::default()
						.name(&(canvas_data.rx_display))
						.marker(if app_state.use_dot { Marker::Dot } else { Marker::Braille })
						.style(Style::default().fg(Color::LightBlue))
						.data(&canvas_data.network_data_rx),
					Dataset::default()
						.name(&(canvas_data.tx_display))
						.marker(if app_state.use_dot { Marker::Dot } else { Marker::Braille })
						.style(Style::default().fg(Color::LightYellow))
						.data(&canvas_data.network_data_tx),
				])
				.render(&mut f, bottom_chunks[0]);
		}

		// Processes table
		{
			let width = f64::from(bottom_chunks[1].width);

			// Admittedly this is kinda a hack... but we need to:
			// * Scroll
			// * Show/hide elements based on scroll position
			// As such, we use a process_counter to know when we've hit the process we've currently scrolled to.  We also need to move the list - we can
			// do so by hiding some elements!
			let num_rows = i64::from(bottom_chunks[1].height) - 3;
			let mut process_counter = 0;

			let start_position = match app_state.scroll_direction {
				app::ScrollDirection::DOWN => {
					if app_state.currently_selected_process_position < num_rows {
						0
					}
					else if app_state.currently_selected_process_position - num_rows < app_state.previous_process_position {
						app_state.previous_process_position
					}
					else {
						app_state.previous_process_position = app_state.currently_selected_process_position - num_rows + 1;
						app_state.previous_process_position
					}
				}
				app::ScrollDirection::UP => {
					if app_state.currently_selected_process_position == app_state.previous_process_position - 1 {
						app_state.previous_process_position = if app_state.previous_process_position > 0 {
							app_state.previous_process_position - 1
						}
						else {
							0
						};
						app_state.previous_process_position
					}
					else {
						app_state.previous_process_position
					}
				}
			};

			/*debug!(
				"START POSN: {}, PREV POSN: {}, CURRENT SELECTED POSN: {}, NUM ROWS: {}",
				start_position, app_state.previous_process_position, app_state.currently_selected_process_position, num_rows
			);*/

			let sliced_vec : Vec<Vec<String>> = (&canvas_data.process_data[start_position as usize..]).to_vec();

			let process_rows = sliced_vec.iter().map(|process| {
				Row::StyledData(
					process.iter(),
					if process_counter == app_state.currently_selected_process_position - start_position {
						// TODO: This is what controls the highlighting!
						process_counter = -1;
						Style::default().fg(Color::Black).bg(Color::Cyan)
					}
					else {
						if process_counter >= 0 {
							process_counter += 1;
						}
						Style::default().fg(TEXT_COLOUR)
					},
				)
			});

			Table::new(["PID", "Name", "CPU%", "Mem%"].iter(), process_rows)
				.block(
					Block::default()
						.title("Processes")
						.borders(Borders::ALL)
						.border_style(match app_state.current_application_position {
							app::ApplicationPosition::PROCESS => highlighted_border_style,
							_ => border_style,
						}),
				)
				.header_style(Style::default().fg(Color::LightBlue))
				.widths(&[(width * 0.2) as u16, (width * 0.35) as u16, (width * 0.2) as u16, (width * 0.2) as u16])
				.render(&mut f, bottom_chunks[1]);
		}
	})?;

	//debug!("Finished drawing.");

	Ok(())
}
