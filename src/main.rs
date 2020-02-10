#[macro_use]
extern crate log;
#[macro_use]
extern crate clap;
#[macro_use]
extern crate lazy_static;

use serde::Deserialize;

use crossterm::{
	event::{
		poll, read, DisableMouseCapture, EnableMouseCapture, Event as CEvent, KeyCode,
		KeyModifiers, MouseEvent,
	},
	execute,
	style::Print,
	terminal::LeaveAlternateScreen,
	terminal::{disable_raw_mode, enable_raw_mode, EnterAlternateScreen},
};

use std::{
	io::{stdout, Write},
	panic::{self, PanicInfo},
	sync::mpsc,
	thread,
	time::{Duration, Instant},
};
use tui::{backend::CrosstermBackend, Terminal};

pub mod app;
mod utils {
	pub mod error;
	pub mod gen_util;
	pub mod logging;
}
mod canvas;
mod constants;
mod data_conversion;

use app::data_harvester::{self, processes::ProcessSorting};
use constants::*;
use data_conversion::*;
use utils::error::{self, BottomError};

enum Event<I, J> {
	KeyInput(I),
	MouseInput(J),
	Update(Box<data_harvester::Data>),
	Clean,
}

enum ResetEvent {
	Reset,
}

#[derive(Deserialize)]
struct Config {
	flags: Option<ConfigFlags>,
	colors: Option<ConfigColours>,
}

#[derive(Deserialize)]
struct ConfigFlags {
	avg_cpu: Option<bool>,
	dot_marker: Option<bool>,
	temperature_type: Option<String>,
	rate: Option<u64>,
	left_legend: Option<bool>,
	current_usage: Option<bool>,
	group_processes: Option<bool>,
	case_sensitive: Option<bool>,
	whole_word: Option<bool>,
	regex: Option<bool>,
}

#[derive(Deserialize)]
struct ConfigColours {
	table_header_color: Option<String>,
	cpu_core_colors: Option<Vec<String>>,
	ram_color: Option<String>,
	swap_color: Option<String>,
	rx_color: Option<String>,
	tx_color: Option<String>,
	border_color: Option<String>,
	highlighted_border_color: Option<String>,
	text_color: Option<String>,
	selected_text_color: Option<String>,
	selected_bg_color: Option<String>,
	widget_title_color: Option<String>,
	graph_color: Option<String>,
}

fn main() -> error::Result<()> {
	//Parse command line options
	let matches = clap_app!(app =>
		(name: crate_name!())
		(version: crate_version!())
		(author: crate_authors!())
		(about: crate_description!())
		(@arg AVG_CPU: -a --avg_cpu "Enables showing the average CPU usage.")
		(@arg DOT_MARKER: -m --dot_marker "Use a dot marker instead of the default braille marker.")
		(@group TEMPERATURE_TYPE =>
			(@arg KELVIN : -k --kelvin "Sets the temperature type to Kelvin.")
			(@arg FAHRENHEIT : -f --fahrenheit "Sets the temperature type to Fahrenheit.")
			(@arg CELSIUS : -c --celsius "Sets the temperature type to Celsius.  This is the default option.")
		)
		(@arg RATE_MILLIS: -r --rate +takes_value "Sets a refresh rate in milliseconds; the minimum is 250ms, defaults to 1000ms.  Smaller values may take more resources.")
		(@arg LEFT_LEGEND: -l --left_legend "Puts external chart legends on the left side rather than the default right side.")
		(@arg USE_CURR_USAGE: -u --current_usage "Within Linux, sets a process' CPU usage to be based on the total current CPU usage, rather than assuming 100% usage.")
		(@arg CONFIG_LOCATION: -C --config +takes_value "Sets the location of the config file.  Expects a config file in the TOML format.")
		//(@arg BASIC_MODE: -b --basic "Sets bottom to basic mode, not showing graphs and only showing basic tables.")
		(@arg GROUP_PROCESSES: -g --group "Groups processes with the same name together on launch.")
		(@arg CASE_SENSITIVE: -S --case_sensitive "Match case when searching by default.")
		(@arg WHOLE_WORD: -W --whole_word "Match whole word when searching by default.")
		(@arg REGEX_DEFAULT: -R --regex "Use regex in searching by default.")
	)
	.get_matches();

	if cfg!(debug_assertions) {
		utils::logging::init_logger()?;
	}

	let config_path = std::path::Path::new(matches.value_of("CONFIG_LOCATION").unwrap_or(
		if cfg!(target_os = "windows") {
			DEFAULT_WINDOWS_CONFIG_FILE_PATH
		} else {
			DEFAULT_UNIX_CONFIG_FILE_PATH
		},
	));

	let config_string = std::fs::read_to_string(config_path);
	let config_toml: Config = if let Ok(config_str) = config_string {
		toml::from_str(&config_str)?
	} else {
		toml::from_str("")?
	};

	let update_rate_in_milliseconds: u128 = if matches.is_present("RATE_MILLIS") {
		matches
			.value_of("RATE_MILLIS")
			.unwrap_or(&DEFAULT_REFRESH_RATE_IN_MILLISECONDS.to_string())
			.parse::<u128>()?
	} else if let Some(flags) = &config_toml.flags {
		if let Some(rate) = flags.rate {
			rate as u128
		} else {
			constants::DEFAULT_REFRESH_RATE_IN_MILLISECONDS
		}
	} else {
		constants::DEFAULT_REFRESH_RATE_IN_MILLISECONDS
	};

	if update_rate_in_milliseconds < 250 {
		return Err(BottomError::InvalidArg(
			"Please set your update rate to be greater than 250 milliseconds.".to_string(),
		));
	} else if update_rate_in_milliseconds > u128::from(std::u64::MAX) {
		return Err(BottomError::InvalidArg(
			"Please set your update rate to be less than unsigned INT_MAX.".to_string(),
		));
	}

	// Set other settings
	let temperature_type = if matches.is_present("FAHRENHEIT") {
		data_harvester::temperature::TemperatureType::Fahrenheit
	} else if matches.is_present("KELVIN") {
		data_harvester::temperature::TemperatureType::Kelvin
	} else if matches.is_present("CELSIUS") {
		data_harvester::temperature::TemperatureType::Celsius
	} else if let Some(flags) = &config_toml.flags {
		if let Some(temp_type) = &flags.temperature_type {
			// Give lowest priority to config.
			match temp_type.as_str() {
				"fahrenheit" | "f" => data_harvester::temperature::TemperatureType::Fahrenheit,
				"kelvin" | "k" => data_harvester::temperature::TemperatureType::Kelvin,
				"celsius" | "c" => data_harvester::temperature::TemperatureType::Celsius,
				_ => {
					return Err(BottomError::ConfigError(
						"Invalid temperature type.  Please have the value be of the form <kelvin|k|celsius|c|fahrenheit|f>".to_string()
					));
				}
			}
		} else {
			data_harvester::temperature::TemperatureType::Celsius
		}
	} else {
		data_harvester::temperature::TemperatureType::Celsius
	};
	let show_average_cpu = if matches.is_present("AVG_CPU") {
		true
	} else if let Some(flags) = &config_toml.flags {
		if let Some(avg_cpu) = flags.avg_cpu {
			avg_cpu
		} else {
			false
		}
	} else {
		false
	};
	let use_dot = if matches.is_present("DOT_MARKER") {
		true
	} else if let Some(flags) = &config_toml.flags {
		if let Some(dot_marker) = flags.dot_marker {
			dot_marker
		} else {
			false
		}
	} else {
		false
	};
	let left_legend = if matches.is_present("LEFT_LEGEND") {
		true
	} else if let Some(flags) = &config_toml.flags {
		if let Some(left_legend) = flags.left_legend {
			left_legend
		} else {
			false
		}
	} else {
		false
	};

	let use_current_cpu_total = if matches.is_present("USE_CURR_USAGE") {
		true
	} else if let Some(flags) = &config_toml.flags {
		if let Some(current_usage) = flags.current_usage {
			current_usage
		} else {
			false
		}
	} else {
		false
	};

	// Create "app" struct, which will control most of the program and store settings/state
	let mut app = app::App::new(
		show_average_cpu,
		temperature_type,
		update_rate_in_milliseconds as u64,
		use_dot,
		left_legend,
		use_current_cpu_total,
	);

	// Enable grouping immediately if set.
	if matches.is_present("GROUP_PROCESSES") {
		app.toggle_grouping();
	} else if let Some(flags) = &config_toml.flags {
		if let Some(grouping) = flags.group_processes {
			if grouping {
				app.toggle_grouping();
			}
		}
	}

	// Set default search method
	if matches.is_present("CASE_SENSITIVE") {
		app.search_state.toggle_ignore_case();
	} else if let Some(flags) = &config_toml.flags {
		if let Some(case_sensitive) = flags.case_sensitive {
			if case_sensitive {
				app.search_state.toggle_ignore_case();
			}
		}
	}

	if matches.is_present("WHOLE_WORD") {
		app.search_state.toggle_search_whole_word();
	} else if let Some(flags) = &config_toml.flags {
		if let Some(whole_word) = flags.whole_word {
			if whole_word {
				app.search_state.toggle_search_whole_word();
			}
		}
	}

	if matches.is_present("REGEX_DEFAULT") {
		app.search_state.toggle_search_regex();
	} else if let Some(flags) = &config_toml.flags {
		if let Some(regex) = flags.regex {
			if regex {
				app.search_state.toggle_search_regex();
			}
		}
	}

	// Set up up tui and crossterm
	let mut stdout_val = stdout();
	enable_raw_mode()?;
	execute!(stdout_val, EnterAlternateScreen)?;
	execute!(stdout_val, EnableMouseCapture)?;

	let mut terminal = Terminal::new(CrosstermBackend::new(stdout_val))?;
	terminal.hide_cursor()?;
	terminal.clear()?;

	// Set panic hook
	panic::set_hook(Box::new(|info| panic_hook(info)));

	// Set up input handling
	let (tx, rx) = mpsc::channel();
	{
		let tx = tx.clone();
		thread::spawn(move || loop {
			if poll(Duration::from_millis(20)).is_ok() {
				let mut mouse_timer = Instant::now();
				let mut keyboard_timer = Instant::now();

				loop {
					if poll(Duration::from_millis(20)).is_ok() {
						if let Ok(event) = read() {
							if let CEvent::Key(key) = event {
								if Instant::now().duration_since(keyboard_timer).as_millis() >= 20 {
									if tx.send(Event::KeyInput(key)).is_err() {
										return;
									}
									keyboard_timer = Instant::now();
								}
							} else if let CEvent::Mouse(mouse) = event {
								if Instant::now().duration_since(mouse_timer).as_millis() >= 20 {
									if tx.send(Event::MouseInput(mouse)).is_err() {
										return;
									}
									mouse_timer = Instant::now();
								}
							}
						}
					}
				}
			}
		});
	}

	// Cleaning loop
	{
		let tx = tx.clone();
		thread::spawn(move || loop {
			thread::sleep(Duration::from_millis(
				constants::STALE_MAX_MILLISECONDS as u64 + 5000,
			));
			tx.send(Event::Clean).unwrap();
		});
	}
	// Event loop
	let (rtx, rrx) = mpsc::channel();
	{
		let tx = tx;
		let temp_type = app.temperature_type.clone();
		thread::spawn(move || {
			let tx = tx.clone();
			let mut data_state = data_harvester::DataState::default();
			data_state.init();
			data_state.set_temperature_type(temp_type);
			data_state.set_use_current_cpu_total(use_current_cpu_total);
			loop {
				if let Ok(message) = rrx.try_recv() {
					match message {
						ResetEvent::Reset => {
							data_state.data.first_run_cleanup();
						}
					}
				}
				futures::executor::block_on(data_state.update_data());
				let event = Event::Update(Box::from(data_state.data.clone()));
				tx.send(event).unwrap();
				thread::sleep(Duration::from_millis(update_rate_in_milliseconds as u64));
			}
		});
	}

	let mut painter = canvas::Painter::default();
	if let Err(config_check) = generate_config_colours(&config_toml, &mut painter) {
		cleanup_terminal(&mut terminal)?;
		return Err(config_check);
	}
	painter.colours.generate_remaining_cpu_colours();
	painter.initialize();

	loop {
		// TODO: [OPT] this should not block...
		if let Ok(recv) = rx.recv_timeout(Duration::from_millis(TICK_RATE_IN_MILLISECONDS)) {
			match recv {
				Event::KeyInput(event) => {
					if event.modifiers.is_empty() {
						// If only a code, and no modifiers, don't bother...

						// Required catch for searching - otherwise you couldn't search with q.
						if event.code == KeyCode::Char('q') && !app.is_in_search_widget() {
							break;
						}

						match event.code {
							KeyCode::End => app.skip_to_last(),
							KeyCode::Home => app.skip_to_first(),
							KeyCode::Up => app.on_up_key(),
							KeyCode::Down => app.on_down_key(),
							KeyCode::Left => app.on_left_key(),
							KeyCode::Right => app.on_right_key(),
							KeyCode::Char(character) => app.on_char_key(character),
							KeyCode::Esc => app.on_esc(),
							KeyCode::Enter => app.on_enter(),
							KeyCode::Tab => app.on_tab(),
							KeyCode::Backspace => app.on_backspace(),
							_ => {}
						}
					} else {
						// Otherwise, track the modifier as well...
						if let KeyModifiers::CONTROL = event.modifiers {
							match event.code {
								KeyCode::Char('c') => break,
								KeyCode::Char('f') => app.enable_searching(),
								KeyCode::Left => app.move_left(),
								KeyCode::Right => app.move_right(),
								KeyCode::Up => app.move_up(),
								KeyCode::Down => app.move_down(),
								KeyCode::Char('r') => {
									if rtx.send(ResetEvent::Reset).is_ok() {
										app.reset();
									}
								}
								KeyCode::Char('a') => app.skip_cursor_beginning(),
								KeyCode::Char('e') => app.skip_cursor_end(),
								_ => {}
							}
						} else if let KeyModifiers::SHIFT = event.modifiers {
							match event.code {
								KeyCode::Left => app.move_left(),
								KeyCode::Right => app.move_right(),
								KeyCode::Up => app.move_up(),
								KeyCode::Down => app.move_down(),
								_ => {}
							}
						} else if let KeyModifiers::ALT = event.modifiers {
							match event.code {
								KeyCode::Char('c') => {
									if app.is_in_search_widget() {
										app.search_state.toggle_ignore_case();
										app.update_regex();
									}
								}
								KeyCode::Char('w') => {
									if app.is_in_search_widget() {
										app.search_state.toggle_search_whole_word();
										app.update_regex();
									}
								}
								KeyCode::Char('r') => {
									if app.is_in_search_widget() {
										app.search_state.toggle_search_regex();
										app.update_regex();
									}
								}
								_ => {}
							}
						}
					}

					if app.update_process_gui {
						update_final_process_list(&mut app);
						app.update_process_gui = false;
					}
				}
				Event::MouseInput(event) => match event {
					MouseEvent::ScrollUp(_x, _y, _modifiers) => app.decrement_position_count(),
					MouseEvent::ScrollDown(_x, _y, _modifiers) => app.increment_position_count(),
					_ => {}
				},
				Event::Update(data) => {
					if !app.is_frozen {
						app.data_collection.eat_data(&data);

						// Convert all data into tui-compliant components

						// Network
						let network_data = convert_network_data_points(&app.data_collection);
						app.canvas_data.network_data_rx = network_data.rx;
						app.canvas_data.network_data_tx = network_data.tx;
						app.canvas_data.rx_display = network_data.rx_display;
						app.canvas_data.tx_display = network_data.tx_display;
						app.canvas_data.total_rx_display = network_data.total_rx_display;
						app.canvas_data.total_tx_display = network_data.total_tx_display;

						// Disk
						app.canvas_data.disk_data = convert_disk_row(&app.data_collection);

						// Temperatures
						app.canvas_data.temp_sensor_data = convert_temp_row(&app);
						// Memory
						app.canvas_data.mem_data = convert_mem_data_points(&app.data_collection);
						app.canvas_data.swap_data = convert_swap_data_points(&app.data_collection);
						let memory_and_swap_labels = convert_mem_labels(&app.data_collection);
						app.canvas_data.mem_label = memory_and_swap_labels.0;
						app.canvas_data.swap_label = memory_and_swap_labels.1;

						// CPU
						app.canvas_data.cpu_data =
							convert_cpu_data_points(app.show_average_cpu, &app.data_collection);

						// Processes
						let (single, grouped) = convert_process_data(&app.data_collection);
						app.canvas_data.process_data = single;
						app.canvas_data.grouped_process_data = grouped;
						update_final_process_list(&mut app);
					}
				}
				Event::Clean => {
					app.data_collection
						.clean_data(constants::STALE_MAX_MILLISECONDS);
				}
			}
		}

		// Quick fix for tab updating the table headers
		if let data_harvester::processes::ProcessSorting::PID = &app.process_sorting_type {
			if app.is_grouped() {
				app.process_sorting_type = data_harvester::processes::ProcessSorting::CPU; // Go back to default, negate PID for group
				app.process_sorting_reverse = true;
			}
		}

		try_drawing(&mut terminal, &mut app, &mut painter)?;
	}

	cleanup_terminal(&mut terminal)?;
	Ok(())
}

fn try_drawing(
	terminal: &mut tui::terminal::Terminal<tui::backend::CrosstermBackend<std::io::Stdout>>,
	app: &mut app::App, painter: &mut canvas::Painter,
) -> error::Result<()> {
	if let Err(err) = painter.draw_data(terminal, app) {
		cleanup_terminal(terminal)?;
		error!("{}", err);
		return Err(err);
	}

	Ok(())
}

fn cleanup_terminal(
	terminal: &mut tui::terminal::Terminal<tui::backend::CrosstermBackend<std::io::Stdout>>,
) -> error::Result<()> {
	disable_raw_mode()?;
	execute!(terminal.backend_mut(), LeaveAlternateScreen)?;
	execute!(terminal.backend_mut(), DisableMouseCapture)?;
	terminal.show_cursor()?;

	Ok(())
}

fn generate_config_colours(
	config_toml: &Config, painter: &mut canvas::Painter,
) -> error::Result<()> {
	if let Some(colours) = &config_toml.colors {
		if let Some(border_color) = &colours.border_color {
			painter.colours.set_border_colour(border_color)?;
		}

		if let Some(highlighted_border_color) = &colours.highlighted_border_color {
			painter
				.colours
				.set_highlighted_border_colour(highlighted_border_color)?;
		}

		if let Some(text_color) = &colours.text_color {
			painter.colours.set_text_colour(text_color)?;
		}

		if let Some(cpu_core_colors) = &(colours.cpu_core_colors) {
			painter.colours.set_cpu_colours(cpu_core_colors)?;
		}

		if let Some(ram_color) = &colours.ram_color {
			painter.colours.set_ram_colour(ram_color)?;
		}

		if let Some(swap_color) = &colours.swap_color {
			painter.colours.set_swap_colour(swap_color)?;
		}

		if let Some(rx_color) = &colours.rx_color {
			painter.colours.set_rx_colour(rx_color)?;
		}

		if let Some(tx_color) = &colours.tx_color {
			painter.colours.set_tx_colour(tx_color)?;
		}

		if let Some(table_header_color) = &colours.table_header_color {
			painter
				.colours
				.set_table_header_colour(table_header_color)?;
		}

		if let Some(scroll_entry_text_color) = &colours.selected_text_color {
			painter
				.colours
				.set_scroll_entry_text_color(scroll_entry_text_color)?;
		}

		if let Some(scroll_entry_bg_color) = &colours.selected_bg_color {
			painter
				.colours
				.set_scroll_entry_bg_color(scroll_entry_bg_color)?;
		}

		if let Some(widget_title_color) = &colours.widget_title_color {
			painter
				.colours
				.set_widget_title_colour(widget_title_color)?;
		}

		if let Some(graph_color) = &colours.graph_color {
			painter.colours.set_graph_colour(graph_color)?;
		}
	}

	Ok(())
}

/// Based on https://github.com/Rigellute/spotify-tui/blob/master/src/main.rs
fn panic_hook(panic_info: &PanicInfo<'_>) {
	let mut stdout = stdout();

	if cfg!(debug_assertions) {
		let msg = match panic_info.payload().downcast_ref::<&'static str>() {
			Some(s) => *s,
			None => match panic_info.payload().downcast_ref::<String>() {
				Some(s) => &s[..],
				None => "Box<Any>",
			},
		};

		let stacktrace: String = format!("{:?}", backtrace::Backtrace::new());

		execute!(
			stdout,
			Print(format!(
				"thread '<unnamed>' panicked at '{}', {}\n\r{}",
				msg,
				panic_info.location().unwrap(),
				stacktrace
			)),
		)
		.unwrap();
	}

	disable_raw_mode().unwrap();
	execute!(stdout, LeaveAlternateScreen).unwrap();
	execute!(stdout, DisableMouseCapture).unwrap();
}

fn update_final_process_list(app: &mut app::App) {
	let mut filtered_process_data: Vec<ConvertedProcessData> = if app.is_grouped() {
		app.canvas_data
			.grouped_process_data
			.clone()
			.into_iter()
			.filter(|process| {
				if let Ok(matcher) = app.get_current_regex_matcher() {
					matcher.is_match(&process.name)
				} else {
					true
				}
			})
			.collect::<Vec<ConvertedProcessData>>()
	} else {
		app.canvas_data
			.process_data
			.iter()
			.filter(|(_pid, process)| {
				if let Ok(matcher) = app.get_current_regex_matcher() {
					if app.search_state.is_searching_with_pid() {
						matcher.is_match(&process.pid.to_string())
					} else {
						matcher.is_match(&process.name)
					}
				} else {
					true
				}
			})
			.map(|(_pid, process)| ConvertedProcessData {
				pid: process.pid,
				name: process.name.clone(),
				cpu_usage: process.cpu_usage_percent,
				mem_usage: process.mem_usage_percent,
				group_pids: vec![process.pid],
			})
			.collect::<Vec<ConvertedProcessData>>()
	};

	sort_process_data(&mut filtered_process_data, app);
	app.canvas_data.finalized_process_data = filtered_process_data;
}

fn sort_process_data(to_sort_vec: &mut Vec<ConvertedProcessData>, app: &app::App) {
	to_sort_vec.sort_by(|a, b| utils::gen_util::get_ordering(&a.name, &b.name, false));

	match app.process_sorting_type {
		ProcessSorting::CPU => {
			to_sort_vec.sort_by(|a, b| {
				utils::gen_util::get_ordering(a.cpu_usage, b.cpu_usage, app.process_sorting_reverse)
			});
		}
		ProcessSorting::MEM => {
			to_sort_vec.sort_by(|a, b| {
				utils::gen_util::get_ordering(a.mem_usage, b.mem_usage, app.process_sorting_reverse)
			});
		}
		ProcessSorting::NAME => to_sort_vec.sort_by(|a, b| {
			utils::gen_util::get_ordering(&a.name, &b.name, app.process_sorting_reverse)
		}),
		ProcessSorting::PID => {
			if !app.is_grouped() {
				to_sort_vec.sort_by(|a, b| {
					utils::gen_util::get_ordering(a.pid, b.pid, app.process_sorting_reverse)
				});
			}
		}
	}
}
