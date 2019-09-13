#![feature(async_closure)]
use crossterm::{input, AlternateScreen, InputEvent, KeyEvent};
use std::{io, sync::mpsc, thread, time::Duration};
use tui::{backend::CrosstermBackend, Terminal};

mod app;
use app::data_collection;

mod utils {
	pub mod error;
	pub mod logging;
}

mod canvas;

#[macro_use]
extern crate log;

enum Event<I> {
	Input(I),
	Update(Box<app::Data>),
}

const STALE_MAX_MILLISECONDS : u64 = 60 * 1000;

#[tokio::main]
async fn main() -> Result<(), io::Error> {
	let screen = AlternateScreen::to_alternate(true)?;
	let backend = CrosstermBackend::with_alternate_screen(screen)?;
	let mut terminal = Terminal::new(backend)?;

	let tick_rate_in_milliseconds : u64 = 250;
	let update_rate_in_milliseconds : u64 = 500; // TODO: Must set a check to prevent this from going into negatives!

	let mut app = app::App::new("rustop");

	let _log = utils::logging::init_logger();

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
	let mut data_state = app::DataState::default();
	data_state.init();
	data_state.set_stale_max_seconds(STALE_MAX_MILLISECONDS);
	data_state.set_temperature_type(app.temperature_type.clone());
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

	let mut app_data = app::Data::default();
	let mut canvas_data = canvas::CanvasData::default();

	loop {
		if let Ok(recv) = rx.recv_timeout(Duration::from_millis(tick_rate_in_milliseconds)) {
			match recv {
				Event::Input(event) => {
					debug!("Input event fired!");
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
						data_collection::processes::sort_processes(&mut app_data.list_of_processes, &app.process_sorting_type, app.process_sorting_reverse);
						canvas_data.process_data = update_process_row(&app_data);
						app.to_be_resorted = false;
					}
					debug!("Input event complete.");
				}
				Event::Update(data) => {
					debug!("Update event fired!");
					app_data = *data;
					data_collection::processes::sort_processes(&mut app_data.list_of_processes, &app.process_sorting_type, app.process_sorting_reverse);

					// Convert all data into tui components
					canvas_data.disk_data = update_disk_row(&app_data);
					canvas_data.temp_sensor_data = update_temp_row(&app_data, &app.temperature_type);
					canvas_data.process_data = update_process_row(&app_data);
					canvas_data.mem_data = update_mem_data_points(&app_data);
					canvas_data.swap_data = update_swap_data_points(&app_data);
					canvas_data.cpu_data = update_cpu_data_points(&app_data);

					debug!("Update event complete.");
				}
			}
			if app.should_quit {
				break;
			}
		}
		// Draw!
		canvas::draw_data(&mut terminal, &canvas_data)?;
	}

	Ok(())
}

fn update_temp_row(app_data : &app::Data, temp_type : &app::TemperatureType) -> Vec<Vec<String>> {
	let mut sensor_vector : Vec<Vec<String>> = Vec::new();

	for sensor in &app_data.list_of_temperature_sensor {
		sensor_vector.push(vec![
			sensor.component_name.to_string(),
			sensor.temperature.to_string()
				+ match temp_type {
					app::TemperatureType::Celsius => "C",
					app::TemperatureType::Kelvin => "K",
					app::TemperatureType::Fahrenheit => "F",
				},
		]);
	}

	sensor_vector
}

fn update_disk_row(app_data : &app::Data) -> Vec<Vec<String>> {
	let mut disk_vector : Vec<Vec<String>> = Vec::new();
	for disk in &app_data.list_of_disks {
		disk_vector.push(vec![
			disk.name.to_string(),
			disk.mount_point.to_string(),
			format!("{:.1}%", disk.used_space as f64 / disk.total_space as f64 * 100_f64),
			(disk.free_space / 1024).to_string() + "GB",
			(disk.total_space / 1024).to_string() + "GB",
		]);
	}

	disk_vector
}

fn update_process_row(app_data : &app::Data) -> Vec<Vec<String>> {
	let mut process_vector : Vec<Vec<String>> = Vec::new();

	for process in &app_data.list_of_processes {
		process_vector.push(vec![
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
		]);
	}

	process_vector
}

fn update_cpu_data_points(app_data : &app::Data) -> Vec<(String, Vec<(f64, f64)>)> {
	let mut cpu_data_vector : Vec<(String, Vec<(f64, f64)>)> = Vec::new();
	let mut cpu_collection : Vec<Vec<(f64, f64)>> = Vec::new();

	if !app_data.list_of_cpu_packages.is_empty() {
		// Initially, populate the cpu_collection.  We want to inject elements in between if possible.

		for cpu_num in 1..app_data.list_of_cpu_packages.last().unwrap().cpu_vec.len() {
			// TODO: 1 to skip total cpu?  Or no?
			let mut this_cpu_data : Vec<(f64, f64)> = Vec::new();

			for data in &app_data.list_of_cpu_packages {
				let current_time = std::time::Instant::now();
				let current_cpu_usage = data.cpu_vec[cpu_num].cpu_usage;
				this_cpu_data.push((
					((STALE_MAX_MILLISECONDS as f64 - current_time.duration_since(data.instant).as_millis() as f64) * 10_f64).floor(),
					current_cpu_usage,
				));
			}

			cpu_collection.push(this_cpu_data);
		}

		// Finally, add it all onto the end
		for (i, data) in cpu_collection.iter().enumerate() {
			cpu_data_vector.push((
				// + 1 to skip total CPU...
				(&*(app_data.list_of_cpu_packages.last().unwrap().cpu_vec[i + 1].cpu_name)).to_string() + " " + &format!("{:3}%", (data.last().unwrap_or(&(0_f64, 0_f64)).1.round() as u64)),
				data.clone(),
			))
		}
	}

	cpu_data_vector
}

fn update_mem_data_points(app_data : &app::Data) -> Vec<(f64, f64)> {
	convert_mem_data(&app_data.memory)
}

fn update_swap_data_points(app_data : &app::Data) -> Vec<(f64, f64)> {
	convert_mem_data(&app_data.swap)
}

fn convert_mem_data(mem_data : &[app::data_collection::mem::MemData]) -> Vec<(f64, f64)> {
	let mut result : Vec<(f64, f64)> = Vec::new();

	for data in mem_data {
		let current_time = std::time::Instant::now();

		result.push((
			((STALE_MAX_MILLISECONDS as f64 - current_time.duration_since(data.instant).as_millis() as f64) * 10_f64).floor(),
			data.mem_used_in_mb as f64 / data.mem_total_in_mb as f64 * 100_f64,
		));
		debug!("Pushed: ({}, {})", result.last().unwrap().0, result.last().unwrap().1);
	}

	result
}
