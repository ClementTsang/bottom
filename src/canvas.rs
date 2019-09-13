use std::io;
use tui::{
	layout::{Constraint, Direction, Layout},
	style::{Color, Modifier, Style},
	widgets::{Axis, Block, Borders, Chart, Dataset, Marker, Row, Table, Widget},
	Terminal,
};

const COLOUR_LIST : [Color; 6] = [Color::LightRed, Color::LightGreen, Color::LightYellow, Color::LightBlue, Color::LightCyan, Color::LightMagenta];
const TEXT_COLOUR : Color = Color::Gray;
const GRAPH_COLOUR : Color = Color::Gray;
const BORDER_STYLE_COLOUR : Color = Color::Gray;

#[derive(Default)]
pub struct CanvasData {
	pub disk_data : Vec<Vec<String>>,
	pub temp_sensor_data : Vec<Vec<String>>,
	pub process_data : Vec<Vec<String>>,
	pub mem_data : Vec<(f64, f64)>,
	pub swap_data : Vec<(f64, f64)>,
	pub cpu_data : Vec<(String, Vec<(f64, f64)>)>,
}

// TODO: Change the error
pub fn draw_data<B : tui::backend::Backend>(terminal : &mut Terminal<B>, canvas_data : &CanvasData) -> Result<(), io::Error> {
	let border_style : Style = Style::default().fg(BORDER_STYLE_COLOUR);

	let temperature_rows = canvas_data.temp_sensor_data.iter().map(|sensor| Row::StyledData(sensor.iter(), Style::default().fg(TEXT_COLOUR)));
	let disk_rows = canvas_data.disk_data.iter().map(|disk| Row::StyledData(disk.iter(), Style::default().fg(TEXT_COLOUR)));
	let process_rows = canvas_data.process_data.iter().map(|process| Row::StyledData(process.iter(), Style::default().fg(TEXT_COLOUR)));

	// TODO: Convert this into a separate func!
	terminal.draw(|mut f| {
		debug!("Drawing!");
		let vertical_chunks = Layout::default()
			.direction(Direction::Vertical)
			.margin(1)
			.constraints([Constraint::Percentage(34), Constraint::Percentage(34), Constraint::Percentage(32)].as_ref())
			.split(f.size());
		let _top_chunks = Layout::default()
			.direction(Direction::Horizontal)
			.margin(0)
			.constraints([Constraint::Percentage(50), Constraint::Percentage(50)].as_ref())
			.split(vertical_chunks[0]);

		let middle_chunks = Layout::default()
			.direction(Direction::Horizontal)
			.margin(0)
			.constraints([Constraint::Percentage(75), Constraint::Percentage(25)].as_ref())
			.split(vertical_chunks[1]);
		let _middle_divided_chunk_1 = Layout::default()
			.direction(Direction::Vertical)
			.margin(0)
			.constraints([Constraint::Percentage(50), Constraint::Percentage(50)].as_ref())
			.split(middle_chunks[0]);
		let _middle_divided_chunk_2 = Layout::default()
			.direction(Direction::Vertical)
			.margin(0)
			.constraints([Constraint::Percentage(50), Constraint::Percentage(50)].as_ref())
			.split(middle_chunks[1]);

		let bottom_chunks = Layout::default()
			.direction(Direction::Horizontal)
			.margin(0)
			.constraints([Constraint::Percentage(50), Constraint::Percentage(50)].as_ref())
			.split(vertical_chunks[2]);
		let bottom_divided_chunk_1 = Layout::default()
			.direction(Direction::Vertical)
			.margin(0)
			.constraints([Constraint::Percentage(50), Constraint::Percentage(50)].as_ref())
			.split(bottom_chunks[0]);
		let bottom_divided_chunk_1_1 = Layout::default()
			.direction(Direction::Horizontal)
			.margin(0)
			.constraints([Constraint::Percentage(40), Constraint::Percentage(60)].as_ref())
			.split(bottom_divided_chunk_1[0]);
		let bottom_divided_chunk_1_2 = Layout::default()
			.direction(Direction::Horizontal)
			.margin(0)
			.constraints([Constraint::Percentage(40), Constraint::Percentage(60)].as_ref())
			.split(bottom_divided_chunk_1[1]);

		// Set up blocks and their components

		// CPU usage graph
		{
			let x_axis : Axis<String> = Axis::default().style(Style::default().fg(GRAPH_COLOUR)).bounds([0.0, 600_000.0]);
			let y_axis = Axis::default().style(Style::default().fg(GRAPH_COLOUR)).bounds([0.0, 100.0]).labels(&["0.0", "50.0", "100.0"]);

			let mut dataset_vector : Vec<Dataset> = Vec::new();
			for (i, cpu) in canvas_data.cpu_data.iter().enumerate() {
				dataset_vector.push(
					Dataset::default()
						.name(&cpu.0)
						.marker(Marker::Dot)
						.style(Style::default().fg(COLOUR_LIST[i % COLOUR_LIST.len()]))
						.data(&(cpu.1)),
				);
			}

			Chart::default()
				.block(Block::default().title("CPU Usage").borders(Borders::ALL).border_style(border_style))
				.x_axis(x_axis)
				.y_axis(y_axis)
				.datasets(&dataset_vector)
				.render(&mut f, vertical_chunks[0]);
		}

		//Memory usage graph
		{
			let x_axis : Axis<String> = Axis::default().style(Style::default().fg(GRAPH_COLOUR)).bounds([0.0, 600_000.0]);
			let y_axis = Axis::default().style(Style::default().fg(GRAPH_COLOUR)).bounds([0.0, 100.0]).labels(&["0", "50", "100"]);
			Chart::default()
				.block(Block::default().title("Memory Usage").borders(Borders::ALL).border_style(border_style))
				.x_axis(x_axis)
				.y_axis(y_axis)
				.datasets(&[
					Dataset::default()
						.name(&("MEM :".to_string() + &format!("{:3}%", (canvas_data.mem_data.last().unwrap_or(&(0_f64, 0_f64)).1.round() as u64))))
						.marker(Marker::Dot)
						.style(Style::default().fg(Color::LightRed))
						.data(&canvas_data.mem_data),
					Dataset::default()
						.name(&("SWAP:".to_string() + &format!("{:3}%", (canvas_data.swap_data.last().unwrap_or(&(0_f64, 0_f64)).1.round() as u64))))
						.marker(Marker::Dot)
						.style(Style::default().fg(Color::LightGreen))
						.data(&canvas_data.swap_data),
				])
				.render(&mut f, middle_chunks[0]);
		}

		// Network graph
		Block::default().title("Network").borders(Borders::ALL).border_style(border_style).render(&mut f, middle_chunks[1]);

		// Temperature table
		Table::new(["Sensor", "Temperature"].iter(), temperature_rows)
			.block(Block::default().title("Temperatures").borders(Borders::ALL).border_style(border_style))
			.header_style(Style::default().fg(Color::LightBlue))
			.widths(&[15, 5])
			.render(&mut f, bottom_divided_chunk_1_1[0]);

		// Disk usage table
		Table::new(["Disk", "Mount", "Used", "Total", "Free"].iter(), disk_rows)
			.block(Block::default().title("Disk Usage").borders(Borders::ALL).border_style(border_style))
			.header_style(Style::default().fg(Color::LightBlue).modifier(Modifier::BOLD))
			.widths(&[15, 10, 5, 5, 5])
			.render(&mut f, bottom_divided_chunk_1_2[0]);

		// Temp graph
		Block::default()
			.title("Temperatures")
			.borders(Borders::ALL)
			.border_style(border_style)
			.render(&mut f, bottom_divided_chunk_1_1[1]);

		// IO graph
		Block::default()
			.title("IO Usage")
			.borders(Borders::ALL)
			.border_style(border_style)
			.render(&mut f, bottom_divided_chunk_1_2[1]);

		// Processes table
		Table::new(["PID", "Name", "CPU%", "Mem%"].iter(), process_rows)
			.block(Block::default().title("Processes").borders(Borders::ALL).border_style(border_style))
			.header_style(Style::default().fg(Color::LightBlue))
			.widths(&[5, 15, 10, 10])
			.render(&mut f, bottom_chunks[1]);
	})?;

	Ok(())
}
