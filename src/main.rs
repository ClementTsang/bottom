#![feature(async_closure)]

#[macro_use]
extern crate log;
#[macro_use]
extern crate clap;
#[macro_use]
extern crate failure;

use crossterm::{input, AlternateScreen, InputEvent, KeyEvent};
use std::{sync::mpsc, thread, time::Duration};
use tui::{backend::CrosstermBackend, Terminal};

mod app;
use app::data_collection;
mod utils {
	pub mod error;
	pub mod logging;
}
use utils::error::{self, RustopError};
mod canvas;

// End imports

enum Event<I> {
	Input(I),
	Update(Box<data_collection::Data>),
}

const STALE_MAX_MILLISECONDS : u64 = 60 * 1000; // We wish to store at most 60 seconds worth of data.  This may change in the future, or be configurable.
const TICK_RATE_IN_MILLISECONDS : u64 = 200; // We use this as it's a good value to work with.

fn main() -> error::Result<()> {
	let _log = utils::logging::init_logger(); // TODO: Error handling

	// Parse command line options
	let matches = clap_app!(app =>
	(name: "rustop")
	(version: crate_version!())
	(author: "Clement Tsang <clementjhtsang@gmail.com>")
	(about: "A graphical top clone.")
	(@arg THEME: -t --theme +takes_value "Sets a colour theme.")
	(@arg AVG_CPU: -a --avgcpu "Enables showing the average CPU usage.")
	(@arg DEBUG: -d --debug "Enables debug mode.") // TODO: This isn't done yet!
	(@group TEMPERATURE_TYPE =>
		(@arg CELSIUS : -c --celsius "Sets the temperature type to Celsius.  This is the default option.")
		(@arg FAHRENHEIT : -f --fahrenheit "Sets the temperature type to Fahrenheit.")
		(@arg KELVIN : -k --kelvin "Sets the temperature type to Kelvin.")
	)
	(@arg RATE: -r --rate +takes_value "Sets a refresh rate in milliseconds, min is 250ms, defaults to 1000ms.  Higher values may take more resources.")
	)
	.after_help("Themes:")
	.get_matches();

	let update_rate_in_milliseconds : u128 = matches.value_of("RATE").unwrap_or("1000").parse::<u128>()?;

	if update_rate_in_milliseconds < 250 {
		return Err(RustopError::InvalidArg {
			message : "Please set your update rate to be greater than 250 milliseconds.".to_string(),
		});
	}
	else if update_rate_in_milliseconds > u128::from(std::u64::MAX) {
		return Err(RustopError::InvalidArg {
			message : "Please set your update rate to be less than unsigned INT_MAX.".to_string(),
		});
	}

	let temperature_type = if matches.is_present("FAHRENHEIT") {
		data_collection::temperature::TemperatureType::Fahrenheit
	}
	else if matches.is_present("KELVIN") {
		data_collection::temperature::TemperatureType::Kelvin
	}
	else {
		data_collection::temperature::TemperatureType::Celsius
	};
	let show_average_cpu = matches.is_present("AVG_CPU");

	// Create "app" struct, which will control most of the program and store settings/state
	let mut app = app::App::new(show_average_cpu, temperature_type, update_rate_in_milliseconds as u64);

	// Set up input handling
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
	let mut data_state = data_collection::DataState::default();
	data_state.init();
	data_state.set_stale_max_seconds(STALE_MAX_MILLISECONDS);
	data_state.set_temperature_type(app.temperature_type.clone());
	{
		let tx = tx.clone();
		let mut first_run = true;
		thread::spawn(move || {
			let tx = tx.clone();
			loop {
				futures::executor::block_on(data_state.update_data());
				tx.send(Event::Update(Box::from(data_state.data.clone()))).unwrap();
				if first_run {
					// Fix for if you set a really long time for update periods (and just gives a faster first value)
					thread::sleep(Duration::from_millis(250));
					first_run = false;
				}
				else {
					thread::sleep(Duration::from_millis(update_rate_in_milliseconds as u64));
				}
			}
		});
	}

	// Set up up tui and crossterm
	let screen = AlternateScreen::to_alternate(true)?;
	let stdout = std::io::stdout();
	let backend = CrosstermBackend::with_alternate_screen(stdout, screen)?;
	let mut terminal = Terminal::new(backend)?;
	terminal.hide_cursor()?;
	terminal.clear()?;

	let mut app_data = data_collection::Data::default();
	let mut canvas_data = canvas::CanvasData::default();

	loop {
		if let Ok(recv) = rx.recv_timeout(Duration::from_millis(TICK_RATE_IN_MILLISECONDS)) {
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
						KeyEvent::Esc => break,
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
					let network_data = update_network_data_points(&app_data);
					canvas_data.network_data_rx = network_data.rx;
					canvas_data.network_data_tx = network_data.tx;
					canvas_data.rx_display = network_data.rx_display;
					canvas_data.tx_display = network_data.tx_display;
					canvas_data.disk_data = update_disk_row(&app_data);
					canvas_data.temp_sensor_data = update_temp_row(&app_data, &app.temperature_type);
					canvas_data.process_data = update_process_row(&app_data);
					canvas_data.mem_data = update_mem_data_points(&app_data);
					canvas_data.swap_data = update_swap_data_points(&app_data);
					canvas_data.cpu_data = update_cpu_data_points(app.show_average_cpu, &app_data);

					debug!("Update event complete.");
				}
			}
			if app.should_quit {
				break;
			}
		}
		// Draw!
		canvas::draw_data(&mut terminal, &app, &canvas_data)?;
	}

	debug!("Terminating.");
	Ok(())
}

fn update_temp_row(app_data : &data_collection::Data, temp_type : &data_collection::temperature::TemperatureType) -> Vec<Vec<String>> {
	let mut sensor_vector : Vec<Vec<String>> = Vec::new();

	for sensor in &app_data.list_of_temperature_sensor {
		sensor_vector.push(vec![
			sensor.component_name.to_string(),
			(sensor.temperature.ceil() as u64).to_string()
				+ match temp_type {
					data_collection::temperature::TemperatureType::Celsius => "C",
					data_collection::temperature::TemperatureType::Kelvin => "K",
					data_collection::temperature::TemperatureType::Fahrenheit => "F",
				},
		]);
	}

	sensor_vector
}

// TODO: IO count
fn update_disk_row(app_data : &data_collection::Data) -> Vec<Vec<String>> {
	let mut disk_vector : Vec<Vec<String>> = Vec::new();
	for disk in &app_data.list_of_disks {
		disk_vector.push(vec![
			disk.name.to_string(),
			disk.mount_point.to_string(),
			format!("{:.1}%", disk.used_space as f64 / disk.total_space as f64 * 100_f64),
			if disk.free_space < 1024 {
				disk.free_space.to_string() + "MB"
			}
			else {
				(disk.free_space / 1024).to_string() + "GB"
			},
			if disk.total_space < 1024 {
				disk.total_space.to_string() + "MB"
			}
			else {
				(disk.total_space / 1024).to_string() + "GB"
			},
		]);
	}

	disk_vector
}

fn update_process_row(app_data : &data_collection::Data) -> Vec<Vec<String>> {
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

fn update_cpu_data_points(show_avg_cpu : bool, app_data : &data_collection::Data) -> Vec<(String, Vec<(f64, f64)>)> {
	let mut cpu_data_vector : Vec<(String, Vec<(f64, f64)>)> = Vec::new();
	let mut cpu_collection : Vec<Vec<(f64, f64)>> = Vec::new();

	if !app_data.list_of_cpu_packages.is_empty() {
		// I'm sorry for the if statement but I couldn't be bothered here...
		for cpu_num in (if show_avg_cpu { 0 } else { 1 })..app_data.list_of_cpu_packages.last().unwrap().cpu_vec.len() {
			let mut this_cpu_data : Vec<(f64, f64)> = Vec::new();

			for data in &app_data.list_of_cpu_packages {
				let current_time = std::time::Instant::now();
				let current_cpu_usage = data.cpu_vec[cpu_num].cpu_usage;

				let new_entry = (
					((STALE_MAX_MILLISECONDS as f64 - current_time.duration_since(data.instant).as_millis() as f64) * 10_f64).floor(),
					current_cpu_usage,
				);

				// Now, inject our joining points...
				if !this_cpu_data.is_empty() {
					let previous_element_data = *(this_cpu_data.last().unwrap());
					for idx in 0..100 {
						this_cpu_data.push((
							previous_element_data.0 + ((new_entry.0 - previous_element_data.0) / 100.0 * f64::from(idx)),
							previous_element_data.1 + ((new_entry.1 - previous_element_data.1) / 100.0 * f64::from(idx)),
						));
					}
				}

				this_cpu_data.push(new_entry);
			}

			cpu_collection.push(this_cpu_data);
		}

		// Finally, add it all onto the end
		for (i, data) in cpu_collection.iter().enumerate() {
			cpu_data_vector.push((
				// + 1 to skip total CPU if show_avg_cpu is false
				format!(
					"{:4}: ",
					&*(app_data.list_of_cpu_packages.last().unwrap().cpu_vec[i + if show_avg_cpu { 0 } else { 1 }].cpu_name)
				)
				.to_uppercase() + &format!("{:3}%", (data.last().unwrap_or(&(0_f64, 0_f64)).1.round() as u64)),
				data.clone(),
			))
		}
	}

	cpu_data_vector
}

fn update_mem_data_points(app_data : &data_collection::Data) -> Vec<(f64, f64)> {
	convert_mem_data(&app_data.memory)
}

fn update_swap_data_points(app_data : &data_collection::Data) -> Vec<(f64, f64)> {
	convert_mem_data(&app_data.swap)
}

fn convert_mem_data(mem_data : &[data_collection::mem::MemData]) -> Vec<(f64, f64)> {
	let mut result : Vec<(f64, f64)> = Vec::new();

	for data in mem_data {
		let current_time = std::time::Instant::now();
		let new_entry = (
			((STALE_MAX_MILLISECONDS as f64 - current_time.duration_since(data.instant).as_millis() as f64) * 10_f64).floor(),
			data.mem_used_in_mb as f64 / data.mem_total_in_mb as f64 * 100_f64,
		);

		// Now, inject our joining points...
		if !result.is_empty() {
			let previous_element_data = *(result.last().unwrap());
			for idx in 0..100 {
				result.push((
					previous_element_data.0 + ((new_entry.0 - previous_element_data.0) / 100.0 * f64::from(idx)),
					previous_element_data.1 + ((new_entry.1 - previous_element_data.1) / 100.0 * f64::from(idx)),
				));
			}
		}

		result.push(new_entry);
		//debug!("Pushed: ({}, {})", result.last().unwrap().0, result.last().unwrap().1);
	}

	result
}

struct ConvertedNetworkData {
	rx : Vec<(f64, f64)>,
	tx : Vec<(f64, f64)>,
	rx_display : String,
	tx_display : String,
}

fn update_network_data_points(app_data : &data_collection::Data) -> ConvertedNetworkData {
	convert_network_data_points(&app_data.network)
}

fn convert_network_data_points(network_data : &[data_collection::network::NetworkData]) -> ConvertedNetworkData {
	let mut rx : Vec<(f64, f64)> = Vec::new();
	let mut tx : Vec<(f64, f64)> = Vec::new();

	for data in network_data {
		let current_time = std::time::Instant::now();
		let rx_data = (
			((STALE_MAX_MILLISECONDS as f64 - current_time.duration_since(data.instant).as_millis() as f64) * 10_f64).floor(),
			data.rx as f64 / 1024.0,
		);
		let tx_data = (
			((STALE_MAX_MILLISECONDS as f64 - current_time.duration_since(data.instant).as_millis() as f64) * 10_f64).floor(),
			data.tx as f64 / 1024.0,
		);

		// Now, inject our joining points...
		if !rx.is_empty() {
			let previous_element_data = *(rx.last().unwrap());
			for idx in 0..100 {
				rx.push((
					previous_element_data.0 + ((rx_data.0 - previous_element_data.0) / 100.0 * f64::from(idx)),
					previous_element_data.1 + ((rx_data.1 - previous_element_data.1) / 100.0 * f64::from(idx)),
				));
			}
		}

		// Now, inject our joining points...
		if !tx.is_empty() {
			let previous_element_data = *(tx.last().unwrap());
			for idx in 0..100 {
				tx.push((
					previous_element_data.0 + ((tx_data.0 - previous_element_data.0) / 100.0 * f64::from(idx)),
					previous_element_data.1 + ((tx_data.1 - previous_element_data.1) / 100.0 * f64::from(idx)),
				));
			}
		}

		rx.push(rx_data);
		tx.push(tx_data);

		debug!("Pushed rx: ({}, {})", rx.last().unwrap().0, rx.last().unwrap().1);
		debug!("Pushed tx: ({}, {})", tx.last().unwrap().0, tx.last().unwrap().1);
	}

	let rx_display = if network_data.is_empty() {
		"0B".to_string()
	}
	else {
		let num_bytes = network_data.last().unwrap().rx;
		if num_bytes < 1024 {
			format!("RX: {:4} B", num_bytes).to_string()
		}
		else if num_bytes < (1024 * 1024) {
			format!("RX: {:4}KB", num_bytes / 1024).to_string()
		}
		else if num_bytes < (1024 * 1024 * 1024) {
			format!("RX: {:4}MB", num_bytes / 1024 / 1024).to_string()
		}
		else {
			format!("RX: {:4}GB", num_bytes / 1024 / 1024 / 1024).to_string()
		}
	};
	let tx_display = if network_data.is_empty() {
		"0B".to_string()
	}
	else {
		let num_bytes = network_data.last().unwrap().tx;
		if num_bytes < 1024 {
			format!("TX: {:4} B", num_bytes).to_string()
		}
		else if num_bytes < (1024 * 1024) {
			format!("TX: {:4}KB", num_bytes / 1024).to_string()
		}
		else if num_bytes < (1024 * 1024 * 1024) {
			format!("TX: {:4}MB", num_bytes / 1024 / 1024).to_string()
		}
		else {
			format!("TX: {:4}GB", num_bytes / 1024 / 1024 / 1024).to_string()
		}
	};

	ConvertedNetworkData { rx, tx, rx_display, tx_display }
}
