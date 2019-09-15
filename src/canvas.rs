use tui::{
	layout::{Constraint, Direction, Layout},
	style::{Color, Modifier, Style},
	widgets::{Axis, Block, Borders, Chart, Dataset, Marker, Row, Table, Widget},
	Terminal,
};

use crate::utils::error;

const COLOUR_LIST : [Color; 6] = [Color::Red, Color::Green, Color::LightYellow, Color::LightBlue, Color::LightCyan, Color::LightMagenta];
const TEXT_COLOUR : Color = Color::Gray;
const GRAPH_COLOUR : Color = Color::Gray;
const BORDER_STYLE_COLOUR : Color = Color::Gray;
const GRAPH_MARKER : Marker = Marker::Braille;

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

pub fn draw_data<B : tui::backend::Backend>(terminal : &mut Terminal<B>, canvas_data : &CanvasData) -> error::Result<()> {
	let border_style : Style = Style::default().fg(BORDER_STYLE_COLOUR);

	let temperature_rows = canvas_data.temp_sensor_data.iter().map(|sensor| Row::StyledData(sensor.iter(), Style::default().fg(TEXT_COLOUR)));
	let disk_rows = canvas_data.disk_data.iter().map(|disk| Row::StyledData(disk.iter(), Style::default().fg(TEXT_COLOUR)));
	let process_rows = canvas_data.process_data.iter().map(|process| Row::StyledData(process.iter(), Style::default().fg(TEXT_COLOUR)));

	terminal.draw(|mut f| {
		debug!("Drawing!");
		let vertical_chunks = Layout::default()
			.direction(Direction::Vertical)
			.margin(1)
			.constraints([Constraint::Percentage(32), Constraint::Percentage(34), Constraint::Percentage(34)].as_ref())
			.split(f.size());

		let middle_chunks = Layout::default()
			.direction(Direction::Horizontal)
			.margin(0)
			.constraints([Constraint::Percentage(65), Constraint::Percentage(35)].as_ref())
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
			let y_axis = Axis::default().style(Style::default().fg(GRAPH_COLOUR)).bounds([-0.5, 100.0]).labels(&["0%", "100%"]);

			let mut dataset_vector : Vec<Dataset> = Vec::new();
			for (i, cpu) in canvas_data.cpu_data.iter().enumerate() {
				dataset_vector.push(
					Dataset::default()
						.name(&cpu.0)
						.marker(GRAPH_MARKER)
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
			let y_axis = Axis::default().style(Style::default().fg(GRAPH_COLOUR)).bounds([-0.5, 100.0]).labels(&["0%", "100%"]); // Offset as the zero value isn't drawn otherwise...
			Chart::default()
				.block(Block::default().title("Memory Usage").borders(Borders::ALL).border_style(border_style))
				.x_axis(x_axis)
				.y_axis(y_axis)
				.datasets(&[
					Dataset::default()
						.name(&("RAM:".to_string() + &format!("{:3}%", (canvas_data.mem_data.last().unwrap_or(&(0_f64, 0_f64)).1.round() as u64))))
						.marker(GRAPH_MARKER)
						.style(Style::default().fg(Color::LightBlue))
						.data(&canvas_data.mem_data),
					Dataset::default()
						.name(&("SWP:".to_string() + &format!("{:3}%", (canvas_data.swap_data.last().unwrap_or(&(0_f64, 0_f64)).1.round() as u64))))
						.marker(GRAPH_MARKER)
						.style(Style::default().fg(Color::LightYellow))
						.data(&canvas_data.swap_data),
				])
				.render(&mut f, middle_chunks[0]);
		}

		// Temperature table
		{
			let width = f64::from(middle_divided_chunk_2[0].width);
			Table::new(["Sensor", "Temp"].iter(), temperature_rows)
				.block(Block::default().title("Temperatures").borders(Borders::ALL).border_style(border_style))
				.header_style(Style::default().fg(Color::LightBlue))
				.widths(&[(width * 0.45) as u16, (width * 0.4) as u16])
				.render(&mut f, middle_divided_chunk_2[0]);
		}

		// Disk usage table
		{
			let width = f64::from(middle_divided_chunk_2[1].width);
			Table::new(["Disk", "Mount", "Used", "Total", "Free"].iter(), disk_rows)
				.block(Block::default().title("Disk Usage").borders(Borders::ALL).border_style(border_style))
				.header_style(Style::default().fg(Color::LightBlue).modifier(Modifier::BOLD))
				.widths(&[(width * 0.25) as u16, (width * 0.2) as u16, (width * 0.15) as u16, (width * 0.15) as u16, (width * 0.15) as u16])
				.render(&mut f, middle_divided_chunk_2[1]);
		}

		// Network graph
		{
			let x_axis : Axis<String> = Axis::default().style(Style::default().fg(GRAPH_COLOUR)).bounds([0.0, 600_000.0]);
			let y_axis = Axis::default().style(Style::default().fg(GRAPH_COLOUR)).bounds([-0.5, 1_000_000.0]).labels(&["0GB", "1GB"]);
			Chart::default()
				.block(Block::default().title("Network").borders(Borders::ALL).border_style(border_style))
				.x_axis(x_axis)
				.y_axis(y_axis)
				.datasets(&[
					Dataset::default()
						.name(&(canvas_data.rx_display))
						.marker(GRAPH_MARKER)
						.style(Style::default().fg(Color::LightBlue))
						.data(&canvas_data.network_data_rx),
					Dataset::default()
						.name(&(canvas_data.tx_display))
						.marker(GRAPH_MARKER)
						.style(Style::default().fg(Color::LightYellow))
						.data(&canvas_data.network_data_tx),
				])
				.render(&mut f, bottom_chunks[0]);
		}

		// Processes table
		{
			let width = f64::from(bottom_chunks[1].width);
			Table::new(["PID", "Name", "CPU%", "Mem%"].iter(), process_rows)
				.block(Block::default().title("Processes").borders(Borders::ALL).border_style(border_style))
				.header_style(Style::default().fg(Color::LightBlue))
				.widths(&[(width * 0.2) as u16, (width * 0.35) as u16, (width * 0.2) as u16, (width * 0.2) as u16])
				.render(&mut f, bottom_chunks[1]);
		}
	})?;

	Ok(())
}
