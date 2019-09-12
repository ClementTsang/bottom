use std::io;
use tui::{
	layout::{Constraint, Direction, Layout},
	style::{Color, Style},
	widgets::{Axis, Block, Borders, Chart, Dataset, Marker, Row, Table, Widget},
	Terminal,
};

const COLOUR_LIST : [Color; 6] = [Color::LightCyan, Color::LightMagenta, Color::LightRed, Color::LightGreen, Color::LightYellow, Color::LightBlue];

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
	let temperature_rows = canvas_data.temp_sensor_data.iter().map(|sensor| {
		Row::StyledData(
			sensor.iter(), // TODO: Change this based on temperature type
			Style::default().fg(Color::White),
		)
	});

	let disk_rows = canvas_data.disk_data.iter().map(|disk| {
		Row::StyledData(
			disk.iter(), // TODO: Change this based on temperature type
			Style::default().fg(Color::White),
		)
	});

	let process_rows = canvas_data.process_data.iter().map(|process| Row::StyledData(process.iter(), Style::default().fg(Color::White)));

	// TODO: Convert this into a separate func!
	terminal.draw(|mut f| {
		let vertical_chunks = Layout::default()
			.direction(Direction::Vertical)
			.margin(1)
			.constraints([Constraint::Percentage(35), Constraint::Percentage(30), Constraint::Percentage(35)].as_ref())
			.split(f.size());
		let top_chunks = Layout::default()
			.direction(Direction::Horizontal)
			.margin(0)
			.constraints([Constraint::Percentage(50), Constraint::Percentage(50)].as_ref())
			.split(vertical_chunks[0]);
		let middle_chunks = Layout::default()
			.direction(Direction::Horizontal)
			.margin(0)
			.constraints([Constraint::Percentage(40), Constraint::Percentage(60)].as_ref())
			.split(vertical_chunks[1]);
		let middle_divided_chunk_1 = Layout::default()
			.direction(Direction::Vertical)
			.margin(0)
			.constraints([Constraint::Percentage(50), Constraint::Percentage(50)].as_ref())
			.split(middle_chunks[0]);
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
			debug!("Drawing CPU...");
			let x_axis : Axis<String> = Axis::default().style(Style::default().fg(Color::White)).bounds([0.0, 60.0]);
			let y_axis = Axis::default().style(Style::default().fg(Color::White)).bounds([0.0, 100.0]).labels(&["0.0", "50.0", "100.0"]);

			let mut dataset_vector : Vec<Dataset> = Vec::new();
			for (i, cpu) in canvas_data.cpu_data.iter().enumerate() {
				dataset_vector.push(
					Dataset::default()
						.name(&cpu.0)
						.marker(Marker::Braille)
						.style(Style::default().fg(COLOUR_LIST[i % COLOUR_LIST.len()]))
						.data(&(cpu.1)),
				);
			}

			Chart::default()
				.block(Block::default().title("CPU Usage").borders(Borders::ALL))
				.x_axis(x_axis)
				.y_axis(y_axis)
				.datasets(&dataset_vector)
				.render(&mut f, top_chunks[0]);
		}

		//Memory usage graph
		{
			let x_axis : Axis<String> = Axis::default().style(Style::default().fg(Color::White)).bounds([0.0, 60.0]);
			let y_axis = Axis::default().style(Style::default().fg(Color::White)).bounds([0.0, 100.0]).labels(&["0.0", "50.0", "100.0"]);
			Chart::default()
				.block(Block::default().title("Memory Usage").borders(Borders::ALL))
				.x_axis(x_axis)
				.y_axis(y_axis)
				.datasets(&[
					Dataset::default()
						.name(&("MEM :".to_string() + &format!("{:3}%", (canvas_data.mem_data.last().unwrap_or(&(0_f64, 0_f64)).1.round() as u64))))
						.marker(Marker::Braille)
						.style(Style::default().fg(Color::Cyan))
						.data(&canvas_data.mem_data),
					Dataset::default()
						.name(&("SWAP:".to_string() + &format!("{:3}%", (canvas_data.swap_data.last().unwrap_or(&(0_f64, 0_f64)).1.round() as u64))))
						.marker(Marker::Braille)
						.style(Style::default().fg(Color::LightGreen))
						.data(&canvas_data.swap_data),
				])
				.render(&mut f, top_chunks[1]);
		}

		// Temperature table
		Table::new(["Sensor", "Temperature"].iter(), temperature_rows)
			.block(Block::default().title("Temperatures").borders(Borders::ALL))
			.header_style(Style::default().fg(Color::LightBlue))
			.widths(&[15, 5])
			.render(&mut f, middle_divided_chunk_1[0]);

		// Disk usage table
		Table::new(["Disk", "Mount", "Used", "Total", "Free"].iter(), disk_rows)
			.block(Block::default().title("Disk Usage").borders(Borders::ALL))
			.header_style(Style::default().fg(Color::LightBlue))
			.widths(&[15, 10, 5, 5, 5])
			.render(&mut f, middle_divided_chunk_1[1]);

		// Temp graph
		Block::default().title("Temperatures").borders(Borders::ALL).render(&mut f, middle_divided_chunk_2[0]);

		// IO graph
		Block::default().title("IO Usage").borders(Borders::ALL).render(&mut f, middle_divided_chunk_2[1]);

		// Network graph
		Block::default().title("Network").borders(Borders::ALL).render(&mut f, bottom_chunks[0]);

		// Processes table
		Table::new(["PID", "Name", "CPU%", "Mem%"].iter(), process_rows)
			.block(Block::default().title("Processes").borders(Borders::ALL))
			.header_style(Style::default().fg(Color::LightBlue))
			.widths(&[5, 15, 10, 10])
			.render(&mut f, bottom_chunks[1]);
	})?;

	Ok(())
}
