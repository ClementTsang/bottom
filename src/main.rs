#![warn(rust_2018_idioms)]

#[macro_use]
extern crate log;
#[macro_use]
extern crate clap;
#[macro_use]
extern crate lazy_static;
#[macro_use]
extern crate futures;

use serde::Deserialize;

use crossterm::{
	event::{
		poll, read, DisableMouseCapture, EnableMouseCapture, Event as CEvent, KeyCode, KeyEvent,
		KeyModifiers, MouseEvent,
	},
	execute,
	style::Print,
	terminal::{disable_raw_mode, enable_raw_mode, EnterAlternateScreen, LeaveAlternateScreen},
};

use std::{
	boxed::Box,
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

use app::{
	data_harvester::{self, processes::ProcessSorting},
	App,
};
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

#[derive(Default, Deserialize)]
struct Config {
	flags : Option<ConfigFlags>,
	colors : Option<ConfigColours>,
}

#[derive(Default, Deserialize)]
struct ConfigFlags {
	avg_cpu : Option<bool>,
	dot_marker : Option<bool>,
	temperature_type : Option<String>,
	rate : Option<u64>,
	left_legend : Option<bool>,
	current_usage : Option<bool>,
	group_processes : Option<bool>,
	case_sensitive : Option<bool>,
	whole_word : Option<bool>,
	regex : Option<bool>,
	default_widget : Option<String>,
	show_disabled_data : Option<bool>,
	//disabled_cpu_cores: Option<Vec<u64>>, // TODO: [FEATURE] Enable disabling cores in config/flags
}

#[derive(Default, Deserialize)]
struct ConfigColours {
	table_header_color : Option<String>,
	avg_cpu_color : Option<String>,
	cpu_core_colors : Option<Vec<String>>,
	ram_color : Option<String>,
	swap_color : Option<String>,
	rx_color : Option<String>,
	tx_color : Option<String>,
	rx_total_color : Option<String>,
	tx_total_color : Option<String>,
	border_color : Option<String>,
	highlighted_border_color : Option<String>,
	text_color : Option<String>,
	selected_text_color : Option<String>,
	selected_bg_color : Option<String>,
	widget_title_color : Option<String>,
	graph_color : Option<String>,
}

fn get_matches() -> clap::ArgMatches<'static> {
	clap_app!(app =>
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
		//(@arg BASIC_MODE: -b --basic "Sets bottom to basic mode, not showing graphs and only showing basic tables.") // TODO: [FEATURE] Min mode
		(@arg GROUP_PROCESSES: -g --group "Groups processes with the same name together on launch.")
		(@arg CASE_SENSITIVE: -S --case_sensitive "Match case when searching by default.")
		(@arg WHOLE_WORD: -W --whole_word "Match whole word when searching by default.")
		(@arg REGEX_DEFAULT: -R --regex "Use regex in searching by default.")
		(@arg SHOW_DISABLED_DATA: -s --show_disabled_data "Show disabled data entries.")
		(@group DEFAULT_WIDGET =>
			(@arg CPU_WIDGET: --cpu_default "Selects the CPU widget to be selected by default.")
			(@arg MEM_WIDGET: --memory_default "Selects the memory widget to be selected by default.")
			(@arg DISK_WIDGET: --disk_default "Selects the disk widget to be selected by default.")
			(@arg TEMP_WIDGET: --temperature_default "Selects the temp widget to be selected by default.")
			(@arg NET_WIDGET: --network_default "Selects the network widget to be selected by default.")
			(@arg PROC_WIDGET: --process_default "Selects the process widget to be selected by default.  This is the default if nothing is set.")
		)
		//(@arg TURNED_OFF_CPUS: -t ... +takes_value "Hides CPU data points by default") // TODO: [FEATURE] Enable disabling cores in config/flags
	)
	.get_matches()
}

#[allow(deprecated)]
fn main() -> error::Result<()> {
	create_logger()?;
	let matches = get_matches();

	let config : Config = create_config(matches.value_of("CONFIG_LOCATION"))?;

	let update_rate_in_milliseconds : u128 =
		get_update_rate_in_milliseconds(&matches.value_of("RATE_MILLIS"), &config)?;

	// Set other settings
	let temperature_type = get_temperature_option(&matches, &config)?;
	let show_average_cpu = get_avg_cpu_option(&matches, &config);
	let use_dot = get_use_dot_option(&matches, &config);
	let left_legend = get_use_left_legend_option(&matches, &config);
	let use_current_cpu_total = get_use_current_cpu_total_option(&matches, &config);
	let current_widget_selected = get_default_widget(&matches, &config);
	let show_disabled_data = get_show_disabled_data_option(&matches, &config);

	// Create "app" struct, which will control most of the program and store settings/state
	let mut app = App::new(
		show_average_cpu,
		temperature_type,
		update_rate_in_milliseconds as u64,
		use_dot,
		left_legend,
		use_current_cpu_total,
		current_widget_selected,
		show_disabled_data,
	);

	enable_app_grouping(&matches, &config, &mut app);
	enable_app_case_sensitive(&matches, &config, &mut app);
	enable_app_match_whole_word(&matches, &config, &mut app);
	enable_app_use_regex(&matches, &config, &mut app);

	// Set up up tui and crossterm
	let mut stdout_val = stdout();
	execute!(stdout_val, EnterAlternateScreen, EnableMouseCapture)?;
	enable_raw_mode()?;

	let mut terminal = Terminal::new(CrosstermBackend::new(stdout_val))?;
	terminal.hide_cursor()?;

	// Set panic hook
	panic::set_hook(Box::new(|info| panic_hook(info)));

	// Set up input handling
	let (tx, rx) = mpsc::channel();
	create_input_thread(tx.clone());

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
	create_event_thread(
		tx,
		rrx,
		use_current_cpu_total,
		update_rate_in_milliseconds as u64,
		app.app_config_fields.temperature_type.clone(),
	);

	let mut painter = canvas::Painter::default();
	if let Err(config_check) = generate_config_colours(&config, &mut painter) {
		cleanup_terminal(&mut terminal)?;
		return Err(config_check);
	}
	painter.colours.generate_remaining_cpu_colours();
	painter.initialize();

	let mut first_run = true;
	loop {
		if let Ok(recv) = rx.recv_timeout(Duration::from_millis(TICK_RATE_IN_MILLISECONDS)) {
			match recv {
				Event::KeyInput(event) => {
					if handle_key_event_or_break(event, &mut app, &rtx) {
						break;
					}

					if app.update_process_gui {
						update_final_process_list(&mut app);
						app.update_process_gui = false;
					}
				}
				Event::MouseInput(event) => handle_mouse_event(event, &mut app),
				Event::Update(data) => {
					app.data_collection.eat_data(&data);

					if !app.is_frozen {
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
						app.canvas_data.cpu_data = convert_cpu_data_points(
							app.app_config_fields.show_average_cpu,
							&app.data_collection,
						);

						// Pre-fill CPU if needed
						if first_run {
							for itx in 0..app.canvas_data.cpu_data.len() {
								if app.cpu_state.core_show_vec.len() <= itx {
									app.cpu_state.core_show_vec.push(true);
								}
							}
							first_run = false;
						}

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

fn handle_mouse_event(event : MouseEvent, app : &mut App) {
	match event {
		MouseEvent::ScrollUp(_x, _y, _modifiers) => app.decrement_position_count(),
		MouseEvent::ScrollDown(_x, _y, _modifiers) => app.increment_position_count(),
		_ => {}
	};
}

fn handle_key_event_or_break(
	event : KeyEvent, app : &mut App, rtx : &std::sync::mpsc::Sender<ResetEvent>,
) -> bool {
	//debug!("KeyEvent: {:?}", event);

	// TODO: [PASTE] Note that this does NOT support some emojis like flags.  This is due to us
	// catching PER CHARACTER right now WITH A forced throttle!  This means multi-char will not work.
	// We can solve this (when we do paste probably) while keeping the throttle (mainly meant for movement)
	// by throttling after *bulk+singular* actions, not just singular ones.

	if event.modifiers.is_empty() {
		// Required catch for searching - otherwise you couldn't search with q.
		if event.code == KeyCode::Char('q') && !app.is_in_search_widget() {
			return true;
		}
		match event.code {
			KeyCode::End => app.skip_to_last(),
			KeyCode::Home => app.skip_to_first(),
			KeyCode::Up => app.on_up_key(),
			KeyCode::Down => app.on_down_key(),
			KeyCode::Left => app.on_left_key(),
			KeyCode::Right => app.on_right_key(),
			KeyCode::Char(caught_char) => app.on_char_key(caught_char),
			KeyCode::Esc => app.on_esc(),
			KeyCode::Enter => app.on_enter(),
			KeyCode::Tab => app.on_tab(),
			KeyCode::Backspace => app.on_backspace(),
			KeyCode::Delete => app.on_delete(),
			KeyCode::F(1) => {
				if app.is_in_search_widget() {
					app.toggle_ignore_case();
				}
			}
			KeyCode::F(2) => {
				if app.is_in_search_widget() {
					app.toggle_search_whole_word();
				}
			}
			KeyCode::F(3) => {
				if app.is_in_search_widget() {
					app.toggle_search_regex();
				}
			}
			_ => {}
		}
	}
	else {
		// Otherwise, track the modifier as well...
		if let KeyModifiers::ALT = event.modifiers {
			match event.code {
				KeyCode::Char('c') | KeyCode::Char('C') => {
					if app.is_in_search_widget() {
						app.toggle_ignore_case();
					}
				}
				KeyCode::Char('w') | KeyCode::Char('W') => {
					if app.is_in_search_widget() {
						app.toggle_search_whole_word();
					}
				}
				KeyCode::Char('r') | KeyCode::Char('R') => {
					if app.is_in_search_widget() {
						app.toggle_search_regex();
					}
				}
				_ => {}
			}
		}
		else if let KeyModifiers::CONTROL = event.modifiers {
			if event.code == KeyCode::Char('c') {
				return true;
			}

			match event.code {
				KeyCode::Char('f') => app.on_slash(),
				KeyCode::Left => app.move_widget_selection_left(),
				KeyCode::Right => app.move_widget_selection_right(),
				KeyCode::Up => app.move_widget_selection_up(),
				KeyCode::Down => app.move_widget_selection_down(),
				KeyCode::Char('r') => {
					if rtx.send(ResetEvent::Reset).is_ok() {
						app.reset();
					}
				}
				KeyCode::Char('u') => app.clear_search(),
				KeyCode::Char('a') => app.skip_cursor_beginning(),
				KeyCode::Char('e') => app.skip_cursor_end(),
				// TODO: [FEATURE] Ctrl-backspace
				// KeyCode::Backspace => app.on_skip_backspace(),
				_ => {}
			}
		}
		else if let KeyModifiers::SHIFT = event.modifiers {
			match event.code {
				KeyCode::Left => app.move_widget_selection_left(),
				KeyCode::Right => app.move_widget_selection_right(),
				KeyCode::Up => app.move_widget_selection_up(),
				KeyCode::Down => app.move_widget_selection_down(),
				KeyCode::Char(caught_char) => app.on_char_key(caught_char),
				_ => {}
			}
		}
	}

	false
}

fn create_logger() -> error::Result<()> {
	if cfg!(debug_assertions) {
		utils::logging::init_logger()?;
	}
	Ok(())
}

fn create_config(flag_config_location : Option<&str>) -> error::Result<Config> {
	use std::ffi::OsString;
	let config_path = if let Some(conf_loc) = flag_config_location {
		OsString::from(conf_loc)
	}
	else if cfg!(target_os = "windows") {
		if let Some(home_path) = dirs::config_dir() {
			let mut path = home_path;
			path.push(DEFAULT_WINDOWS_CONFIG_FILE_PATH);
			path.into_os_string()
		}
		else {
			OsString::new()
		}
	}
	else if let Some(home_path) = dirs::home_dir() {
		let mut path = home_path;
		path.push(DEFAULT_UNIX_CONFIG_FILE_PATH);
		path.into_os_string()
	}
	else {
		OsString::new()
	};

	let path = std::path::Path::new(&config_path);

	if let Ok(config_str) = std::fs::read_to_string(path) {
		Ok(toml::from_str(config_str.as_str())?)
	}
	else {
		Ok(Config::default())
	}
}

fn get_update_rate_in_milliseconds(
	update_rate : &Option<&str>, config : &Config,
) -> error::Result<u128> {
	let update_rate_in_milliseconds = if let Some(update_rate) = update_rate {
		update_rate.parse::<u128>()?
	}
	else if let Some(flags) = &config.flags {
		if let Some(rate) = flags.rate {
			rate as u128
		}
		else {
			constants::DEFAULT_REFRESH_RATE_IN_MILLISECONDS
		}
	}
	else {
		constants::DEFAULT_REFRESH_RATE_IN_MILLISECONDS
	};

	if update_rate_in_milliseconds < 250 {
		return Err(BottomError::InvalidArg(
			"Please set your update rate to be greater than 250 milliseconds.".to_string(),
		));
	}
	else if update_rate_in_milliseconds > u128::from(std::u64::MAX) {
		return Err(BottomError::InvalidArg(
			"Please set your update rate to be less than unsigned INT_MAX.".to_string(),
		));
	}

	Ok(update_rate_in_milliseconds)
}

fn get_temperature_option(
	matches : &clap::ArgMatches<'static>, config : &Config,
) -> error::Result<data_harvester::temperature::TemperatureType> {
	if matches.is_present("FAHRENHEIT") {
		return Ok(data_harvester::temperature::TemperatureType::Fahrenheit);
	}
	else if matches.is_present("KELVIN") {
		return Ok(data_harvester::temperature::TemperatureType::Kelvin);
	}
	else if matches.is_present("CELSIUS") {
		return Ok(data_harvester::temperature::TemperatureType::Celsius);
	}
	else if let Some(flags) = &config.flags {
		if let Some(temp_type) = &flags.temperature_type {
			// Give lowest priority to config.
			match temp_type.as_str() {
				"fahrenheit" | "f" => {
					return Ok(data_harvester::temperature::TemperatureType::Fahrenheit);
				}
				"kelvin" | "k" => {
					return Ok(data_harvester::temperature::TemperatureType::Kelvin);
				}
				"celsius" | "c" => {
					return Ok(data_harvester::temperature::TemperatureType::Celsius);
				}
				_ => {
					return Err(BottomError::ConfigError(
						"Invalid temperature type.  Please have the value be of the form \
						 <kelvin|k|celsius|c|fahrenheit|f>"
							.to_string(),
					));
				}
			}
		}
	}
	Ok(data_harvester::temperature::TemperatureType::Celsius)
}

fn get_avg_cpu_option(matches : &clap::ArgMatches<'static>, config : &Config) -> bool {
	if matches.is_present("AVG_CPU") {
		return true;
	}
	else if let Some(flags) = &config.flags {
		if let Some(avg_cpu) = flags.avg_cpu {
			return avg_cpu;
		}
	}

	false
}

fn get_use_dot_option(matches : &clap::ArgMatches<'static>, config : &Config) -> bool {
	if matches.is_present("DOT_MARKER") {
		return true;
	}
	else if let Some(flags) = &config.flags {
		if let Some(dot_marker) = flags.dot_marker {
			return dot_marker;
		}
	}
	false
}

fn get_use_left_legend_option(matches : &clap::ArgMatches<'static>, config : &Config) -> bool {
	if matches.is_present("LEFT_LEGEND") {
		return true;
	}
	else if let Some(flags) = &config.flags {
		if let Some(left_legend) = flags.left_legend {
			return left_legend;
		}
	}

	false
}

fn get_use_current_cpu_total_option(
	matches : &clap::ArgMatches<'static>, config : &Config,
) -> bool {
	if matches.is_present("USE_CURR_USAGE") {
		return true;
	}
	else if let Some(flags) = &config.flags {
		if let Some(current_usage) = flags.current_usage {
			return current_usage;
		}
	}

	false
}

fn get_show_disabled_data_option(matches : &clap::ArgMatches<'static>, config : &Config) -> bool {
	if matches.is_present("SHOW_DISABLED_DATA") {
		return true;
	}
	else if let Some(flags) = &config.flags {
		if let Some(show_disabled_data) = flags.show_disabled_data {
			return show_disabled_data;
		}
	}

	false
}

fn enable_app_grouping(matches : &clap::ArgMatches<'static>, config : &Config, app : &mut App) {
	if matches.is_present("GROUP_PROCESSES") {
		app.toggle_grouping();
	}
	else if let Some(flags) = &config.flags {
		if let Some(grouping) = flags.group_processes {
			if grouping {
				app.toggle_grouping();
			}
		}
	}
}

fn enable_app_case_sensitive(
	matches : &clap::ArgMatches<'static>, config : &Config, app : &mut App,
) {
	if matches.is_present("CASE_SENSITIVE") {
		app.process_search_state.search_toggle_ignore_case();
	}
	else if let Some(flags) = &config.flags {
		if let Some(case_sensitive) = flags.case_sensitive {
			if case_sensitive {
				app.process_search_state.search_toggle_ignore_case();
			}
		}
	}
}

fn enable_app_match_whole_word(
	matches : &clap::ArgMatches<'static>, config : &Config, app : &mut App,
) {
	if matches.is_present("WHOLE_WORD") {
		app.process_search_state.search_toggle_whole_word();
	}
	else if let Some(flags) = &config.flags {
		if let Some(whole_word) = flags.whole_word {
			if whole_word {
				app.process_search_state.search_toggle_whole_word();
			}
		}
	}
}

fn enable_app_use_regex(matches : &clap::ArgMatches<'static>, config : &Config, app : &mut App) {
	if matches.is_present("REGEX_DEFAULT") {
		app.process_search_state.search_toggle_regex();
	}
	else if let Some(flags) = &config.flags {
		if let Some(regex) = flags.regex {
			if regex {
				app.process_search_state.search_toggle_regex();
			}
		}
	}
}

fn get_default_widget(
	matches : &clap::ArgMatches<'static>, config : &Config,
) -> app::WidgetPosition {
	if matches.is_present("CPU_WIDGET") {
		return app::WidgetPosition::Cpu;
	}
	else if matches.is_present("MEM_WIDGET") {
		return app::WidgetPosition::Mem;
	}
	else if matches.is_present("DISK_WIDGET") {
		return app::WidgetPosition::Disk;
	}
	else if matches.is_present("TEMP_WIDGET") {
		return app::WidgetPosition::Temp;
	}
	else if matches.is_present("NET_WIDGET") {
		return app::WidgetPosition::Network;
	}
	else if matches.is_present("PROC_WIDGET") {
		return app::WidgetPosition::Process;
	}
	else if let Some(flags) = &config.flags {
		if let Some(default_widget) = &flags.default_widget {
			match default_widget.as_str() {
				"cpu_default" => {
					return app::WidgetPosition::Cpu;
				}
				"memory_default" => {
					return app::WidgetPosition::Mem;
				}
				"processes_default" => {
					return app::WidgetPosition::Process;
				}
				"network_default" => {
					return app::WidgetPosition::Network;
				}
				"temperature_default" => {
					return app::WidgetPosition::Temp;
				}
				"disk_default" => {
					return app::WidgetPosition::Disk;
				}
				_ => {}
			}
		}
	}

	app::WidgetPosition::Process
}

fn try_drawing(
	terminal : &mut tui::terminal::Terminal<tui::backend::CrosstermBackend<std::io::Stdout>>,
	app : &mut App, painter : &mut canvas::Painter,
) -> error::Result<()> {
	if let Err(err) = painter.draw_data(terminal, app) {
		cleanup_terminal(terminal)?;
		error!("{}", err);
		return Err(err);
	}

	Ok(())
}

fn cleanup_terminal(
	terminal : &mut tui::terminal::Terminal<tui::backend::CrosstermBackend<std::io::Stdout>>,
) -> error::Result<()> {
	disable_raw_mode()?;
	execute!(
		terminal.backend_mut(),
		DisableMouseCapture,
		LeaveAlternateScreen
	)?;
	terminal.show_cursor()?;

	Ok(())
}

fn generate_config_colours(config : &Config, painter : &mut canvas::Painter) -> error::Result<()> {
	if let Some(colours) = &config.colors {
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

		if let Some(avg_cpu_color) = &colours.avg_cpu_color {
			painter.colours.set_avg_cpu_colour(avg_cpu_color)?;
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

		if let Some(rx_total_color) = &colours.rx_total_color {
			painter.colours.set_rx_total_colour(rx_total_color)?;
		}

		if let Some(tx_total_color) = &colours.tx_total_color {
			painter.colours.set_tx_total_colour(tx_total_color)?;
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
fn panic_hook(panic_info : &PanicInfo<'_>) {
	let mut stdout = stdout();

	let msg = match panic_info.payload().downcast_ref::<&'static str>() {
		Some(s) => *s,
		None => match panic_info.payload().downcast_ref::<String>() {
			Some(s) => &s[..],
			None => "Box<Any>",
		},
	};

	let stacktrace : String = format!("{:?}", backtrace::Backtrace::new());

	disable_raw_mode().unwrap();
	execute!(stdout, LeaveAlternateScreen, DisableMouseCapture).unwrap();

	// Print stack trace.  Must be done after!
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

fn update_final_process_list(app : &mut App) {
	let mut filtered_process_data : Vec<ConvertedProcessData> = if app.is_grouped() {
		app.canvas_data
			.grouped_process_data
			.iter()
			.filter(|process| {
				if app
					.process_search_state
					.search_state
					.is_invalid_or_blank_search()
				{
					return true;
				}
				else if let Some(matcher_result) = app.get_current_regex_matcher() {
					if let Ok(matcher) = matcher_result {
						return matcher.is_match(&process.name);
					}
				}

				true
			})
			.cloned()
			.collect::<Vec<_>>()
	}
	else {
		app.canvas_data
			.process_data
			.iter()
			.filter_map(|(_pid, process)| {
				let mut result = true;

				if !app
					.process_search_state
					.search_state
					.is_invalid_or_blank_search()
				{
					if let Some(matcher_result) = app.get_current_regex_matcher() {
						if let Ok(matcher) = matcher_result {
							if app.process_search_state.is_searching_with_pid {
								result = matcher.is_match(&process.pid.to_string());
							}
							else {
								result = matcher.is_match(&process.name);
							}
						}
					}
				}

				if result {
					return Some(ConvertedProcessData {
						pid : process.pid,
						name : process.name.clone(),
						cpu_usage : process.cpu_usage_percent,
						mem_usage : process.mem_usage_percent,
						group_pids : vec![process.pid],
					});
				}

				None
			})
			.collect::<Vec<_>>()
	};

	sort_process_data(&mut filtered_process_data, app);
	app.canvas_data.finalized_process_data = filtered_process_data;
}

fn sort_process_data(to_sort_vec : &mut Vec<ConvertedProcessData>, app : &App) {
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

fn create_input_thread(
	tx : std::sync::mpsc::Sender<Event<crossterm::event::KeyEvent, crossterm::event::MouseEvent>>,
) {
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
						}
						else if let CEvent::Mouse(mouse) = event {
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

fn create_event_thread(
	tx : std::sync::mpsc::Sender<Event<crossterm::event::KeyEvent, crossterm::event::MouseEvent>>,
	rrx : std::sync::mpsc::Receiver<ResetEvent>, use_current_cpu_total : bool,
	update_rate_in_milliseconds : u64, temp_type : data_harvester::temperature::TemperatureType,
) {
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
			let event = Event::Update(Box::from(data_state.data));
			data_state.data = data_harvester::Data::default();
			tx.send(event).unwrap();
			thread::sleep(Duration::from_millis(update_rate_in_milliseconds));
		}
	});
}
