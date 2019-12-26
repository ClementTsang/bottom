#[macro_use]
extern crate log;
#[macro_use]
extern crate clap;
#[macro_use]
extern crate failure;

use crossterm::{
	event::{self, Event as CEvent, KeyCode, KeyEvent, KeyModifiers, MouseEvent},
	execute,
	terminal::LeaveAlternateScreen,
	terminal::{disable_raw_mode, enable_raw_mode, EnterAlternateScreen},
};

use std::{
	io::{stdout, Write},
	sync::mpsc,
	thread,
	time::{Duration, Instant},
};
use tui::{backend::CrosstermBackend, Terminal};

pub mod app;
mod utils {
	pub mod error;
	pub mod logging;
}
mod canvas;
mod constants;
mod data_conversion;

use app::data_collection;
use constants::TICK_RATE_IN_MILLISECONDS;
use data_conversion::*;
use utils::error::{self, BottomError};

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
	// Parse command line options
	let matches = clap_app!(app =>
		(name: crate_name!())
		(version: crate_version!())
		(author: crate_authors!())
		(about: crate_description!())
		(@arg AVG_CPU: -a --avgcpu "Enables showing the average CPU usage.")
		(@arg DOT_MARKER: -m --dot_marker "Use a dot marker instead of the default braille marker.")
		(@arg DEBUG: -d --debug "Enables debug mode, which will output a log file.")
		(@group TEMPERATURE_TYPE =>
			(@arg CELSIUS : -c --celsius "Sets the temperature type to Celsius.  This is the default option.")
			(@arg FAHRENHEIT : -f --fahrenheit "Sets the temperature type to Fahrenheit.")
			(@arg KELVIN : -k --kelvin "Sets the temperature type to Kelvin.")
		)
		(@arg RATE_MILLIS: -r --rate +takes_value "Sets a refresh rate in milliseconds; the minimum is 250ms, defaults to 1000ms.  Smaller values may take more resources.")
		//(@arg CONFIG_LOCATION: -co --config +takes_value "Sets the location of the config file.  Expects a config file in the JSON format.")
		//(@arg BASIC_MODE: -b --basic "Sets bottom to basic mode, not showing graphs and only showing basic tables.")
	)
	.get_matches();

	let update_rate_in_milliseconds: u128 = if matches.is_present("RATE_MILLIS") {
		matches
			.value_of("RATE_MILLIS")
			.unwrap_or(&constants::DEFAULT_REFRESH_RATE_IN_MILLISECONDS.to_string())
			.parse::<u128>()?
	} else {
		constants::DEFAULT_REFRESH_RATE_IN_MILLISECONDS
	};

	if update_rate_in_milliseconds < 250 {
		return Err(BottomError::InvalidArg {
			message: "Please set your update rate to be greater than 250 milliseconds.".to_string(),
		});
	} else if update_rate_in_milliseconds > u128::from(std::u64::MAX) {
		return Err(BottomError::InvalidArg {
			message: "Please set your update rate to be less than unsigned INT_MAX.".to_string(),
		});
	}

	// Attempt to create debugging...
	let enable_debugging = matches.is_present("DEBUG");
	if enable_debugging || cfg!(debug_assertions) {
		utils::logging::init_logger()?;
	}

	let temperature_type = if matches.is_present("FAHRENHEIT") {
		data_collection::temperature::TemperatureType::Fahrenheit
	} else if matches.is_present("KELVIN") {
		data_collection::temperature::TemperatureType::Kelvin
	} else {
		data_collection::temperature::TemperatureType::Celsius
	};
	let show_average_cpu = matches.is_present("AVG_CPU");
	let use_dot = matches.is_present("DOT_MARKER");

	// Create "app" struct, which will control most of the program and store settings/state
	let mut app = app::App::new(show_average_cpu, temperature_type, update_rate_in_milliseconds as u64, use_dot);

	// Set up up tui and crossterm
	let mut stdout = stdout();
	enable_raw_mode()?;
	execute!(stdout, EnterAlternateScreen)?;

	let backend = CrosstermBackend::new(stdout);

	let mut terminal = Terminal::new(backend)?;
	terminal.hide_cursor()?;
	terminal.clear()?;

	// Set up input handling
	let (tx, rx) = mpsc::channel();
	{
		let tx = tx.clone();
		thread::spawn(move || {
			let mut mouse_timer = Instant::now();
			let mut keyboard_timer = Instant::now();

			loop {
				if let Ok(event) = event::read() {
					if let CEvent::Key(key) = event {
						if Instant::now().duration_since(keyboard_timer).as_millis() >= 30 {
							if tx.send(Event::KeyInput(key)).is_err() {
								return;
							}
							keyboard_timer = Instant::now();
						}
					} else if let CEvent::Mouse(mouse) = event {
						if Instant::now().duration_since(mouse_timer).as_millis() >= 30 {
							if tx.send(Event::MouseInput(mouse)).is_err() {
								return;
							}
							mouse_timer = Instant::now();
						}
					}
				}
			}
		});
	}

	// Event loop
	let (rtx, rrx) = mpsc::channel();
	{
		let tx = tx;
		let mut first_run = true;
		let temp_type = app.temperature_type.clone();
		thread::spawn(move || {
			let tx = tx.clone();
			let mut data_state = data_collection::DataState::default();
			data_state.init();
			data_state.set_temperature_type(temp_type);
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
				} else {
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
					if event.modifiers.is_empty() {
						// If only a code, and no modifiers, don't bother...
						match event.code {
							KeyCode::Char('q') => break,
							KeyCode::Char('G') => app.skip_to_last(),
							KeyCode::Char('h') => app.on_left(),
							KeyCode::Char('l') => app.on_right(),
							KeyCode::Char('k') => app.on_up(),
							KeyCode::Char('j') => app.on_down(),
							KeyCode::Up => app.decrement_position_count(),
							KeyCode::Down => app.increment_position_count(),
							KeyCode::Char(uncaught_char) => app.on_key(uncaught_char),
							KeyCode::Esc => app.on_esc(),
							_ => {}
						}
					} else {
						// Otherwise, track the modifier as well...
						match event {
							KeyEvent {
								modifiers: KeyModifiers::CONTROL,
								code: KeyCode::Char('c'),
							} => break,
							KeyEvent {
								modifiers: KeyModifiers::CONTROL,
								code: KeyCode::Left,
							} => app.on_left(),
							KeyEvent {
								modifiers: KeyModifiers::CONTROL,
								code: KeyCode::Right,
							} => app.on_right(),
							KeyEvent {
								modifiers: KeyModifiers::CONTROL,
								code: KeyCode::Up,
							} => app.on_up(),
							KeyEvent {
								modifiers: KeyModifiers::CONTROL,
								code: KeyCode::Down,
							} => app.on_down(),
							KeyEvent {
								modifiers: KeyModifiers::CONTROL,
								code: KeyCode::Char('r'),
							} => {
								while rtx.send(ResetEvent::Reset).is_err() {
									debug!("Sent reset message.");
								}
								debug!("Resetting begins...");
								app.reset();
							}
							_ => {}
						}
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
				}
				Event::MouseInput(event) => match event {
					MouseEvent::ScrollUp(_x, _y, _modifiers) => app.decrement_position_count(),
					MouseEvent::ScrollDown(_x, _y, _modifiers) => app.increment_position_count(),
					_ => {}
				},
				Event::Update(data) => {
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

						//debug!("Update event complete.");
					}
				}
			}
		}
		// Draw!
		if let Err(err) = canvas::draw_data(&mut terminal, &mut app, &canvas_data) {
			disable_raw_mode()?;
			execute!(terminal.backend_mut(), LeaveAlternateScreen)?;
			terminal.show_cursor()?;
			error!("{}", err);
			return Err(err);
		}
	}

	disable_raw_mode()?;
	execute!(terminal.backend_mut(), LeaveAlternateScreen)?;
	terminal.show_cursor()?;
	Ok(())
}
