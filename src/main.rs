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
	Update(widgets::Data),
}

#[tokio::main]
async fn main() -> Result<(), io::Error> {
	let screen = AlternateScreen::to_alternate(true)?;
	let backend = CrosstermBackend::with_alternate_screen(screen)?;
	let mut terminal = Terminal::new(backend)?;

	let tick_rate_in_milliseconds : u64 = 220;
	let update_rate_in_milliseconds : u64 = 1000;

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
	{
		let tx = tx.clone();
		thread::spawn(move || {
			let tx = tx.clone();
			loop {
				futures::executor::block_on(data_state.update_data()); // TODO: Fix
				tx.send(Event::Update(data_state.data.clone())).unwrap();
				thread::sleep(Duration::from_millis(update_rate_in_milliseconds));
			}
		});
	}

	let mut app = widgets::App::new("rustop");
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
					app_data = data;
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
				format!("{:.2}%", disk.used_space as f64 / disk.total_space as f64 * 100_f64),
				(disk.free_space / 1024).to_string() + "GB",
				(disk.total_space / 1024).to_string() + "GB",
			]
			.into_iter(), // TODO: Change this based on temperature type
			Style::default().fg(Color::LightGreen),
		)
	});

	let mem_total_mb = app_data.memory.mem_total_in_mb as f64;
	let process_rows = app_data.list_of_processes.iter().map(|process| {
		Row::StyledData(
			vec![
				process.pid.to_string(),
				process.command.to_string(),
				format!("{:.2}%", process.cpu_usage_percent),
				format!("{:.2}%", process.mem_usage_in_mb as f64 / mem_total_mb * 100_f64),
			]
			.into_iter(),
			Style::default().fg(Color::LightGreen),
		)
	});

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
		let middle_divided_chunk = Layout::default()
			.direction(Direction::Vertical)
			.margin(0)
			.constraints([Constraint::Percentage(50), Constraint::Percentage(50)].as_ref())
			.split(middle_chunks[0]);
		let bottom_chunks = Layout::default()
			.direction(Direction::Horizontal)
			.margin(0)
			.constraints([Constraint::Percentage(50), Constraint::Percentage(50)].as_ref())
			.split(vertical_chunks[2]);

		// Set up blocks and their components

		// CPU usage graph
		Chart::default()
			.block(Block::default().title("CPU Usage").borders(Borders::ALL))
			.x_axis(Axis::default().style(Style::default().fg(Color::White)).bounds([0.0, 10.0]).labels(&["0.0", "10.0"]))
			.y_axis(Axis::default().style(Style::default().fg(Color::White)).bounds([0.0, 10.0]).labels(&["0.0", "10.0"]))
			.datasets(&[
				Dataset::default()
					.name("data1")
					.marker(Marker::Dot)
					.style(Style::default().fg(Color::Cyan))
					.data(&[(0.0, 5.0), (1.0, 6.0), (1.5, 6.434)]),
				Dataset::default()
					.name("data2")
					.marker(Marker::Braille)
					.style(Style::default().fg(Color::Magenta))
					.data(&[(4.0, 5.0), (5.0, 8.0), (7.66, 13.5)]),
			])
			.render(&mut f, top_chunks[0]);

		//Memory usage graph
		Block::default().title("Memory Usage").borders(Borders::ALL).render(&mut f, top_chunks[1]);

		// Temperature table
		Table::new(["Sensor", "Temperature"].iter(), temperature_rows)
			.block(Block::default().title("Temperatures").borders(Borders::ALL))
			.header_style(Style::default().fg(Color::LightBlue))
			.widths(&[25, 25])
			.render(&mut f, middle_divided_chunk[0]);

		// Disk usage table
		Table::new(["Disk", "Mount", "Used", "Total", "Free"].iter(), disk_rows)
			.block(Block::default().title("Disk Usage").borders(Borders::ALL))
			.header_style(Style::default().fg(Color::LightBlue))
			.widths(&[25, 25, 10, 10, 10])
			.render(&mut f, middle_divided_chunk[1]);

		// IO graph
		Block::default().title("IO Usage").borders(Borders::ALL).render(&mut f, middle_chunks[1]);

		// Network graph
		Block::default().title("Network").borders(Borders::ALL).render(&mut f, bottom_chunks[0]);

		// Processes table
		Table::new(["PID", "Command", "CPU%", "Mem%"].iter(), process_rows)
			.block(Block::default().title("Processes").borders(Borders::ALL))
			.header_style(Style::default().fg(Color::LightBlue))
			.widths(&[5, 15, 10, 10])
			.render(&mut f, bottom_chunks[1]);
	})?;

	Ok(())
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
		.level(log::LevelFilter::Debug)
		.chain(fern::log_file("debug.log")?)
		.apply()?;

	Ok(())
}

fn try_debug(result_log : &Result<(), fern::InitError>, message : &str) {
	if result_log.is_ok() {
		debug!("{}", message);
	}
}
