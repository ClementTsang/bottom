#![feature(async_closure)]
use crossterm::{input, AlternateScreen, InputEvent, KeyEvent};
use std::{io, sync::mpsc, thread, time::Duration};
use tui::{
	backend::CrosstermBackend,
	layout::{Constraint, Direction, Layout},
	style::{Color, Style},
	widgets::{Axis, Block, Borders, Chart, Dataset, Marker, Row, Table, Widget},
	Terminal,
};

mod widgets;

#[macro_use]
extern crate log;

enum Event<I> {
	Input(I),
	Update(Box<widgets::Data>),
}

const STALE_MAX_SECONDS : u64 = 60;

#[tokio::main]
async fn main() -> Result<(), io::Error> {
	let screen = AlternateScreen::to_alternate(true)?;
	let backend = CrosstermBackend::with_alternate_screen(screen)?;
	let mut terminal = Terminal::new(backend)?;

	let tick_rate_in_milliseconds : u64 = 250;
	let update_rate_in_milliseconds : u64 = 1000; // TODO: Must set a check to prevent this from going into negatives!

	let mut app = widgets::App::new("rustop");

	let log = init_logger();

	terminal.hide_cursor()?;
	// Setup input handling
	let (tx, rx) = mpsc::channel();
	{
		let tx = tx.clone();
		thread::spawn(move || {
			let input = input();
			let reader = input.read_sync();
			for event in reader {
				if let InputEvent::Keyboard(key) = event {
					if tx.send(Event::Input(key.clone())).is_err() {
						return;
					}
				}
			}
		});
	}

	// Event loop
	let mut data_state = widgets::DataState::default();
	data_state.init();
	data_state.set_stale_max_seconds(STALE_MAX_SECONDS);
	{
		let tx = tx.clone();
		thread::spawn(move || {
			let tx = tx.clone();
			loop {
				futures::executor::block_on(data_state.update_data()); // TODO: Fix
				tx.send(Event::Update(Box::from(data_state.data.clone()))).unwrap();
				thread::sleep(Duration::from_millis(update_rate_in_milliseconds));
			}
		});
	}

	terminal.clear()?;

	let mut app_data = widgets::Data::default();
	loop {
		if let Ok(recv) = rx.recv_timeout(Duration::from_millis(tick_rate_in_milliseconds)) {
			match recv {
				Event::Input(event) => {
					try_debug(&log, "Input event fired!");
					match event {
						KeyEvent::Char(c) => app.on_key(c),
						KeyEvent::Left => app.on_left(),
						KeyEvent::Right => app.on_right(),
						KeyEvent::Up => app.on_up(),
						KeyEvent::Down => app.on_down(),
						KeyEvent::Ctrl('c') => break,
						_ => {}
					}

					if app.to_be_resorted {
						widgets::processes::sort_processes(&mut app_data.list_of_processes, &app.process_sorting_type, app.process_sorting_reverse);
						app.to_be_resorted = false;
					}
					try_debug(&log, "Input event complete.");
				}
				Event::Update(data) => {
					try_debug(&log, "Update event fired!");
					app_data = *data;
					widgets::processes::sort_processes(&mut app_data.list_of_processes, &app.process_sorting_type, app.process_sorting_reverse);
					try_debug(&log, "Update event complete.");
				}
			}
			if app.should_quit {
				break;
			}
		}

		// Draw!
		draw_data(&mut terminal, &app_data)?;
	}

	Ok(())
}

fn draw_data<B : tui::backend::Backend>(terminal : &mut Terminal<B>, app_data : &widgets::Data) -> Result<(), io::Error> {
	// Convert data into tui components
	let temperature_rows = app_data.list_of_temperature.iter().map(|sensor| {
		Row::StyledData(
			vec![sensor.component_name.to_string(), sensor.temperature.to_string() + "C"].into_iter(), // TODO: Change this based on temperature type
			Style::default().fg(Color::LightGreen),
		)
	});

	let disk_rows = app_data.list_of_disks.iter().map(|disk| {
		Row::StyledData(
			vec![
				disk.name.to_string(),
				disk.mount_point.to_string(),
				format!("{:.1}%", disk.used_space as f64 / disk.total_space as f64 * 100_f64),
				(disk.free_space / 1024).to_string() + "GB",
				(disk.total_space / 1024).to_string() + "GB",
			]
			.into_iter(), // TODO: Change this based on temperature type
			Style::default().fg(Color::LightGreen),
		)
	});

	let process_rows = app_data.list_of_processes.iter().map(|process| {
		Row::StyledData(
			vec![
				process.pid.to_string(),
				process.command.to_string(),
				format!("{:.1}%", process.cpu_usage_percent),
				format!(
					"{:.1}%",
					if let Some(mem_usage) = process.mem_usage_percent {
						mem_usage
					}
					else if let Some(mem_usage_in_mb) = process.mem_usage_mb {
						if let Some(mem_data) = app_data.memory.last() {
							mem_usage_in_mb as f64 / mem_data.mem_total_in_mb as f64 * 100_f64
						}
						else {
							0_f64
						}
					}
					else {
						0_f64
					}
				),
			]
			.into_iter(),
			Style::default().fg(Color::LightGreen),
		)
	});

	// TODO: Convert this into a separate func!
	terminal.draw(|mut f| {
		let vertical_chunks = Layout::default()
			.direction(Direction::Vertical)
			.margin(1)
			.constraints([Constraint::Percentage(30), Constraint::Percentage(40), Constraint::Percentage(30)].as_ref())
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
			let x_axis : Axis<String> = Axis::default().style(Style::default().fg(Color::White)).bounds([0.0, 60.0]);
			let y_axis = Axis::default().style(Style::default().fg(Color::White)).bounds([0.0, 100.0]).labels(&["0.0", "50.0", "100.0"]);
			Chart::default()
				.block(Block::default().title("CPU Usage").borders(Borders::ALL))
				.x_axis(x_axis)
				.y_axis(y_axis)
				.datasets(&[
					Dataset::default()
						.name("CPU0")
						.marker(Marker::Braille)
						.style(Style::default().fg(Color::Cyan))
						.data(&convert_cpu_data(0, &app_data.list_of_cpu_packages)),
					Dataset::default()
						.name("CPU1")
						.marker(Marker::Braille)
						.style(Style::default().fg(Color::LightMagenta))
						.data(&convert_cpu_data(1, &app_data.list_of_cpu_packages)),
				])
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
						.name("MEM")
						.marker(Marker::Braille)
						.style(Style::default().fg(Color::Cyan))
						.data(&convert_mem_data(&app_data.memory)),
					Dataset::default()
						.name("SWAP")
						.marker(Marker::Braille)
						.style(Style::default().fg(Color::LightGreen))
						.data(&convert_mem_data(&app_data.swap)),
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

// TODO: Remove this count, this is for testing, lol
fn convert_cpu_data(count : usize, cpu_data : &[widgets::cpu::CPUPackage]) -> Vec<(f64, f64)> {
	let mut result : Vec<(f64, f64)> = Vec::new();
	let current_time = std::time::Instant::now();

	for data in cpu_data {
		result.push((
			STALE_MAX_SECONDS as f64 - current_time.duration_since(data.instant).as_secs() as f64,
			f64::from(data.cpu_vec[count + 1].cpu_usage),
		));
	}

	result
}

fn convert_mem_data(mem_data : &[widgets::mem::MemData]) -> Vec<(f64, f64)> {
	let mut result : Vec<(f64, f64)> = Vec::new();
	let current_time = std::time::Instant::now();

	for data in mem_data {
		result.push((
			STALE_MAX_SECONDS as f64 - current_time.duration_since(data.instant).as_secs() as f64,
			data.mem_used_in_mb as f64 / data.mem_total_in_mb as f64 * 100_f64,
		));
	}

	result
}

fn init_logger() -> Result<(), fern::InitError> {
	fern::Dispatch::new()
		.format(|out, message, record| {
			out.finish(format_args!(
				"{}[{}][{}] {}",
				chrono::Local::now().format("[%Y-%m-%d][%H:%M:%S]"),
				record.target(),
				record.level(),
				message
			))
		})
		.level(if cfg!(debug_assertions) { log::LevelFilter::Debug } else { log::LevelFilter::Info })
		.chain(fern::log_file("debug.log")?)
		.apply()?;

	Ok(())
}

fn try_debug(result_log : &Result<(), fern::InitError>, message : &str) {
	if result_log.is_ok() {
		debug!("{}", message);
	}
}
