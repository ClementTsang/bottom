#![feature(async_closure)]

#[macro_use]
extern crate log;
#[macro_use]
extern crate clap;
#[macro_use]
extern crate failure;

use crossterm::{input, AlternateScreen, InputEvent, KeyEvent, MouseButton, MouseEvent};
use std::{io::stdout, sync::mpsc, thread, time::Duration};
use tui_temp_fork::{backend::CrosstermBackend, Terminal};

pub mod app;
mod utils {
	pub mod error;
	pub mod logging;
}
mod canvas;
mod constants;
mod convert_data;

use app::data_collection;
use constants::TICK_RATE_IN_MILLISECONDS;
use convert_data::*;
use utils::error::{self, RustopError};

// End imports

enum Event<I, J> {
	KeyInput(I),
	MouseInput(J),
	Update(Box<data_collection::Data>),
}

enum ResetEvent {
	Reset,
}

fn main() -> error::Result<()> {
	utils::logging::init_logger()?;

	// Parse command line options
	let matches = clap_app!(app =>
	(name: crate_name!())
	(version: crate_version!())
	(author: crate_authors!())
	(about: crate_description!())
	//(@arg THEME: -t --theme +takes_value "Sets a colour theme.")
	(@arg AVG_CPU: -a --avgcpu "Enables showing the average CPU usage.")
	(@arg DOT_MARKER: -m --dot_marker "Use a dot marker instead of the default braille marker.  May be needed in things like Powershell.")
	//(@arg DEBUG: -d --debug "Enables debug mode.") // TODO: This isn't done yet!
	(@group TEMPERATURE_TYPE =>
		(@arg CELSIUS : -c --celsius "Sets the temperature type to Celsius.  This is the default option.")
		(@arg FAHRENHEIT : -f --fahrenheit "Sets the temperature type to Fahrenheit.")
		(@arg KELVIN : -k --kelvin "Sets the temperature type to Kelvin.")
	)
	(@arg RATE_MILLIS: -r --rate +takes_value "Sets a refresh rate in milliseconds; the minimum is 250ms, defaults to 1000ms.  Smaller values may take more resources.")
	)
	//.after_help("Themes:") // TODO: This and others disabled for now
	.get_matches();

	let update_rate_in_milliseconds : u128 = if matches.is_present("RATE_MILLIS") {
		matches
			.value_of("RATE_MILLIS")
			.unwrap_or(&constants::DEFAULT_REFRESH_RATE_IN_MILLISECONDS.to_string())
			.parse::<u128>()?
	}
	else {
		constants::DEFAULT_REFRESH_RATE_IN_MILLISECONDS
	};

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
	let use_dot = matches.is_present("DOT_MARKER");

	// Create "app" struct, which will control most of the program and store settings/state
	let mut app = app::App::new(show_average_cpu, temperature_type, update_rate_in_milliseconds as u64, use_dot);

	// Set up up tui and crossterm
	let screen = AlternateScreen::to_alternate(true)?;
	let stdout = stdout();

	let backend = CrosstermBackend::with_alternate_screen(stdout, screen)?;
	let mut terminal = Terminal::new(backend)?;
	terminal.hide_cursor()?;
	terminal.clear()?;

	// Set up input handling
	let (tx, rx) = mpsc::channel();
	{
		let tx = tx.clone();
		thread::spawn(move || {
			let input = input();
			input.enable_mouse_mode().unwrap();

			// TODO: I don't like this... seems odd async doesn't work on linux (then again, sync didn't work on windows)
			if cfg!(target_os = "linux") {
				let reader = input.read_sync();
				for event in reader {
					match event {
						InputEvent::Keyboard(key) => {
							if tx.send(Event::KeyInput(key.clone())).is_err() {
								return;
							}
						}
						InputEvent::Mouse(mouse) => {
							if tx.send(Event::MouseInput(mouse)).is_err() {
								return;
							}
						}
						_ => {}
					}
				}
			}
			else {
				let mut reader = input.read_async();
				loop {
					if let Some(event) = reader.next() {
						match event {
							InputEvent::Keyboard(key) => {
								if tx.send(Event::KeyInput(key.clone())).is_err() {
									return;
								}
							}
							InputEvent::Mouse(mouse) => {
								if tx.send(Event::MouseInput(mouse)).is_err() {
									return;
								}
							}
							_ => {}
						}
					}
				}
			}
		});
	}

	// Event loop
	let mut data_state = data_collection::DataState::default();
	data_state.init();
	data_state.set_temperature_type(app.temperature_type.clone());
	let (rtx, rrx) = mpsc::channel();
	{
		let tx = tx.clone();
		let mut first_run = true;
		thread::spawn(move || {
			let tx = tx.clone();
			loop {
				if let Ok(message) = rrx.try_recv() {
					match message {
						ResetEvent::Reset => {
							debug!("Received reset message");
							first_run = true;
							data_state.data = app::data_collection::Data::default();
						}
					}
				}
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

	let mut canvas_data = canvas::CanvasData::default();
	loop {
		if let Ok(recv) = rx.recv_timeout(Duration::from_millis(TICK_RATE_IN_MILLISECONDS)) {
			match recv {
				Event::KeyInput(event) => {
					// debug!("Keyboard event fired!");
					match event {
						KeyEvent::Ctrl('c') | KeyEvent::Char('q') => break,
						KeyEvent::Char('h') | KeyEvent::Left => app.on_left(),
						KeyEvent::Char('l') | KeyEvent::Right => app.on_right(),
						KeyEvent::Char('k') | KeyEvent::Up => app.on_up(),
						KeyEvent::Char('j') | KeyEvent::Down => app.on_down(),
						KeyEvent::Ctrl('r') => {
							while rtx.send(ResetEvent::Reset).is_err() {
								debug!("Sent reset message.");
							}
							debug!("Resetting begins...");
						}
						KeyEvent::ShiftUp => app.decrement_position_count(),
						KeyEvent::ShiftDown => app.increment_position_count(),
						KeyEvent::Char(c) => app.on_key(c),
						KeyEvent::Enter => app.on_enter(),
						KeyEvent::Esc => app.on_esc(),
						_ => {}
					}

					if app.to_be_resorted {
						data_collection::processes::sort_processes(
							&mut app.data.list_of_processes,
							&app.process_sorting_type,
							app.process_sorting_reverse,
						);
						canvas_data.process_data = update_process_row(&app.data);
						app.to_be_resorted = false;
					}
					// debug!("Input event complete.");
				}
				Event::MouseInput(event) => {
					// debug!("Mouse event fired!");
					match event {
						MouseEvent::Press(e, _x, _y) => match e {
							MouseButton::WheelUp => {
								app.decrement_position_count();
							}
							MouseButton::WheelDown => {
								app.increment_position_count();
							}
							_ => {}
						},
						MouseEvent::Hold(_x, _y) => {}
						MouseEvent::Release(_x, _y) => {}
						_ => {}
					}
				}
				Event::Update(data) => {
					// debug!("Update event fired!");
					if !app.is_frozen {
						app.data = *data;

						data_collection::processes::sort_processes(
							&mut app.data.list_of_processes,
							&app.process_sorting_type,
							app.process_sorting_reverse,
						);

						// Convert all data into tui components
						let network_data = update_network_data_points(&app.data);
						canvas_data.network_data_rx = network_data.rx;
						canvas_data.network_data_tx = network_data.tx;
						canvas_data.rx_display = network_data.rx_display;
						canvas_data.tx_display = network_data.tx_display;
						canvas_data.disk_data = update_disk_row(&app.data);
						canvas_data.temp_sensor_data = update_temp_row(&app.data, &app.temperature_type);
						canvas_data.process_data = update_process_row(&app.data);
						canvas_data.mem_data = update_mem_data_points(&app.data);
						canvas_data.mem_values = update_mem_data_values(&app.data);
						canvas_data.swap_data = update_swap_data_points(&app.data);
						canvas_data.cpu_data = update_cpu_data_points(app.show_average_cpu, &app.data);

						debug!("Update event complete.");
					}
				}
			}
		}
		// Draw!
		if let Err(err) = canvas::draw_data(&mut terminal, &mut app, &canvas_data) {
			input().disable_mouse_mode().unwrap();
			error!("{}", err);
			return Err(err);
		}
	}

	input().disable_mouse_mode().unwrap();
	debug!("Terminating.");
	Ok(())
}
