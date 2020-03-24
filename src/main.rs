#![warn(rust_2018_idioms)]

#[macro_use]
extern crate log;

use std::{
    boxed::Box,
    io::{stdout, Write},
    panic::{self, PanicInfo},
    sync::mpsc,
    thread,
    time::{Duration, Instant},
};

use clap::*;

use crossterm::{
    event::{
        poll, read, DisableMouseCapture, EnableMouseCapture, Event, KeyCode, KeyEvent,
        KeyModifiers, MouseEvent,
    },
    execute,
    style::Print,
    terminal::{disable_raw_mode, enable_raw_mode, EnterAlternateScreen, LeaveAlternateScreen},
};
use tui::{backend::CrosstermBackend, Terminal};

use app::{
    data_harvester::{self, processes::ProcessSorting},
    App,
};
use constants::*;
use data_conversion::*;
use options::*;
use utils::error;

pub mod app;

mod utils {
    pub mod error;
    pub mod gen_util;
    pub mod logging;
}

mod canvas;
mod constants;
mod data_conversion;

pub mod options;

enum BottomEvent<I, J> {
    KeyInput(I),
    MouseInput(J),
    Update(Box<data_harvester::Data>),
    Clean,
}

enum ResetEvent {
    Reset,
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
		(@arg CONFIG_LOCATION: -C --config +takes_value "Sets the location of the config file.  Expects a config file in the TOML format. If it doesn't exist, one is created.")
		(@arg BASIC_MODE: -b --basic "Hides graphs and uses a more basic look")
		(@arg GROUP_PROCESSES: -g --group "Groups processes with the same name together on launch.")
		(@arg CASE_SENSITIVE: -S --case_sensitive "Match case when searching by default.")
		(@arg WHOLE_WORD: -W --whole_word "Match whole word when searching by default.")
		(@arg REGEX_DEFAULT: -R --regex "Use regex in searching by default.")
        (@arg SHOW_DISABLED_DATA: -s --show_disabled_data "Show disabled data entries.")
        (@arg DEFAULT_TIME_VALUE: -t --default_time_value +takes_value "Default time value for graphs in milliseconds; minimum is 30s, defaults to 60s.")
        (@arg TIME_DELTA: -d --time_delta +takes_value "The amount changed upon zooming in/out in milliseconds; minimum is 1s, defaults to 15s.")
        (@arg HIDE_TIME: --hide_time "Completely hide the time scaling")
        (@arg AUTOHIDE_TIME: --autohide_time "Automatically hide the time scaling in graphs after being shown for a brief moment when zoomed in/out.  If time is disabled via --hide_time then this will have no effect.")
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

fn main() -> error::Result<()> {
    create_logger()?;
    let matches = get_matches();

    let config: Config = create_config(matches.value_of("CONFIG_LOCATION"))?;

    // Get widget layout separately
    let widget_layout = get_widget_layout(&config)?;

    // Create "app" struct, which will control most of the program and store settings/state
    let mut app = build_app(&matches, &config, &widget_layout)?;

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
                constants::STALE_MAX_MILLISECONDS + 5000,
            ));
            tx.send(BottomEvent::Clean).unwrap();
        });
    }
    // Event loop
    let (rtx, rrx) = mpsc::channel();
    create_event_thread(
        tx,
        rrx,
        app.app_config_fields.use_current_cpu_total,
        app.app_config_fields.update_rate_in_milliseconds,
        app.app_config_fields.temperature_type.clone(),
        app.app_config_fields.show_average_cpu,
    );

    let mut painter = canvas::Painter::init(widget_layout);
    if let Err(config_check) = generate_config_colours(&config, &mut painter) {
        cleanup_terminal(&mut terminal)?;
        return Err(config_check);
    }
    painter.colours.generate_remaining_cpu_colours();
    painter.complete_painter_init();

    let mut first_run = true;
    loop {
        if let Ok(recv) = rx.recv_timeout(Duration::from_millis(TICK_RATE_IN_MILLISECONDS)) {
            match recv {
                BottomEvent::KeyInput(event) => {
                    if handle_key_event_or_break(event, &mut app, &rtx) {
                        break;
                    }
                    handle_force_redraws(&mut app);
                }
                BottomEvent::MouseInput(event) => {
                    handle_mouse_event(event, &mut app);
                    handle_force_redraws(&mut app);
                }
                BottomEvent::Update(data) => {
                    app.data_collection.eat_data(&data);

                    if !app.is_frozen {
                        // Convert all data into tui-compliant components

                        // Network
                        let network_data = convert_network_data_points(&app.data_collection, false);
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
                        app.canvas_data.mem_data =
                            convert_mem_data_points(&app.data_collection, false);
                        app.canvas_data.swap_data =
                            convert_swap_data_points(&app.data_collection, false);
                        let memory_and_swap_labels = convert_mem_labels(&app.data_collection);
                        app.canvas_data.mem_label = memory_and_swap_labels.0;
                        app.canvas_data.swap_label = memory_and_swap_labels.1;

                        // Pre-fill CPU if needed
                        if first_run {
                            let cpu_len = app.data_collection.cpu_harvest.len();
                            app.cpu_state.widget_states.values_mut().for_each(|state| {
                                state.core_show_vec = vec![true; cpu_len];
                                state.num_cpus_shown = cpu_len;
                            });
                            app.cpu_state.num_cpus_total = cpu_len;
                            first_run = false;
                        }

                        // CPU
                        app.canvas_data.cpu_data =
                            convert_cpu_data_points(&app.data_collection, false);

                        // Processes
                        let (single, grouped) = convert_process_data(&app.data_collection);
                        app.canvas_data.process_data = single;
                        app.canvas_data.grouped_process_data = grouped;
                        update_all_process_lists(&mut app);
                    }
                }
                BottomEvent::Clean => {
                    app.data_collection
                        .clean_data(constants::STALE_MAX_MILLISECONDS);
                }
            }
        }

        try_drawing(&mut terminal, &mut app, &mut painter)?;
    }

    cleanup_terminal(&mut terminal)?;
    Ok(())
}

fn handle_mouse_event(event: MouseEvent, app: &mut App) {
    match event {
        MouseEvent::ScrollUp(_x, _y, _modifiers) => app.handle_scroll_up(),
        MouseEvent::ScrollDown(_x, _y, _modifiers) => app.handle_scroll_down(),
        _ => {}
    };
}

fn handle_key_event_or_break(
    event: KeyEvent, app: &mut App, rtx: &std::sync::mpsc::Sender<ResetEvent>,
) -> bool {
    // debug!("KeyEvent: {:?}", event);

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
                app.toggle_ignore_case();
            }
            KeyCode::F(2) => {
                app.toggle_search_whole_word();
            }
            KeyCode::F(3) => {
                app.toggle_search_regex();
            }
            _ => {}
        }
    } else {
        // Otherwise, track the modifier as well...
        if let KeyModifiers::ALT = event.modifiers {
            match event.code {
                KeyCode::Char('c') | KeyCode::Char('C') => {
                    app.toggle_ignore_case();
                }
                KeyCode::Char('w') | KeyCode::Char('W') => {
                    app.toggle_search_whole_word();
                }
                KeyCode::Char('r') | KeyCode::Char('R') => {
                    app.toggle_search_regex();
                }
                _ => {}
            }
        } else if let KeyModifiers::CONTROL = event.modifiers {
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
                KeyCode::Char('a') => app.skip_cursor_beginning(),
                KeyCode::Char('e') => app.skip_cursor_end(),
                KeyCode::Char('u') => app.clear_search(),
                // KeyCode::Char('j') => {}, // Move down
                // KeyCode::Char('k') => {}, // Move up
                // KeyCode::Char('h') => {}, // Move right
                // KeyCode::Char('l') => {}, // Move left
                // Can't do now, CTRL+BACKSPACE doesn't work and graphemes
                // are hard to iter while truncating last (eloquently).
                // KeyCode::Backspace => app.skip_word_backspace(),
                _ => {}
            }
        } else if let KeyModifiers::SHIFT = event.modifiers {
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

fn create_config(flag_config_location: Option<&str>) -> error::Result<Config> {
    use std::{ffi::OsString, fs};
    let config_path = if let Some(conf_loc) = flag_config_location {
        OsString::from(conf_loc)
    } else if cfg!(target_os = "windows") {
        if let Some(home_path) = dirs::config_dir() {
            let mut path = home_path;
            path.push(DEFAULT_WINDOWS_CONFIG_FILE_PATH);
            path.into_os_string()
        } else {
            OsString::new()
        }
    } else if let Some(home_path) = dirs::home_dir() {
        let mut path = home_path;
        path.push(DEFAULT_UNIX_CONFIG_FILE_PATH);
        path.into_os_string()
    } else {
        OsString::new()
    };

    let path = std::path::Path::new(&config_path);

    if let Ok(config_string) = fs::read_to_string(path) {
        Ok(toml::from_str(config_string.as_str())?)
    } else {
        if let Some(parent_path) = path.parent() {
            fs::create_dir_all(parent_path)?;
        }
        fs::File::create(path)?.write_all(DEFAULT_CONFIG_CONTENT.as_bytes())?;
        Ok(toml::from_str(DEFAULT_CONFIG_CONTENT)?)
    }
}

fn try_drawing(
    terminal: &mut tui::terminal::Terminal<tui::backend::CrosstermBackend<std::io::Stdout>>,
    app: &mut App, painter: &mut canvas::Painter,
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
    execute!(
        terminal.backend_mut(),
        DisableMouseCapture,
        LeaveAlternateScreen
    )?;
    terminal.show_cursor()?;

    Ok(())
}

fn generate_config_colours(config: &Config, painter: &mut canvas::Painter) -> error::Result<()> {
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

        if let Some(cpu_core_colors) = &colours.cpu_core_colors {
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
fn panic_hook(panic_info: &PanicInfo<'_>) {
    let mut stdout = stdout();

    let msg = match panic_info.payload().downcast_ref::<&'static str>() {
        Some(s) => *s,
        None => match panic_info.payload().downcast_ref::<String>() {
            Some(s) => &s[..],
            None => "Box<Any>",
        },
    };

    let stacktrace: String = format!("{:?}", backtrace::Backtrace::new());

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

fn handle_force_redraws(app: &mut App) {
    // Currently we use an Option... because we might want to future-proof this
    // if we eventually get widget-specific redrawing!
    if app.proc_state.force_update_all {
        update_all_process_lists(app);
        app.proc_state.force_update_all = false;
    } else if let Some(widget_id) = app.proc_state.force_update {
        update_final_process_list(app, widget_id);
        app.proc_state.force_update = None;
    }

    if app.cpu_state.force_update.is_some() {
        app.canvas_data.cpu_data = convert_cpu_data_points(&app.data_collection, app.is_frozen);
        app.cpu_state.force_update = None;
    }

    if app.mem_state.force_update.is_some() {
        app.canvas_data.mem_data = convert_mem_data_points(&app.data_collection, app.is_frozen);
        app.canvas_data.swap_data = convert_swap_data_points(&app.data_collection, app.is_frozen);
        app.mem_state.force_update = None;
    }

    if app.net_state.force_update.is_some() {
        let (rx, tx) = get_rx_tx_data_points(&app.data_collection, app.is_frozen);
        app.canvas_data.network_data_rx = rx;
        app.canvas_data.network_data_tx = tx;
        app.net_state.force_update = None;
    }
}

fn update_all_process_lists(app: &mut App) {
    let widget_ids = app
        .proc_state
        .widget_states
        .keys()
        .cloned()
        .collect::<Vec<_>>();

    widget_ids.into_iter().for_each(|widget_id| {
        update_final_process_list(app, widget_id);
    });
}

fn update_final_process_list(app: &mut App, widget_id: u64) {
    let is_invalid_or_blank = match app.proc_state.widget_states.get(&widget_id) {
        Some(process_state) => process_state
            .process_search_state
            .search_state
            .is_invalid_or_blank_search(),
        None => false,
    };

    let filtered_process_data: Vec<ConvertedProcessData> = if app.is_grouped(widget_id) {
        app.canvas_data
            .grouped_process_data
            .iter()
            .filter(|process| {
                if is_invalid_or_blank {
                    return true;
                } else if let Some(matcher_result) = app.get_current_regex_matcher(widget_id) {
                    if let Ok(matcher) = matcher_result {
                        return matcher.is_match(&process.name);
                    }
                }

                true
            })
            .cloned()
            .collect::<Vec<_>>()
    } else {
        let is_searching_with_pid = match app.proc_state.widget_states.get(&widget_id) {
            Some(process_state) => process_state.process_search_state.is_searching_with_pid,
            None => false,
        };

        app.canvas_data
            .process_data
            .iter()
            .filter_map(|(_pid, process)| {
                let mut result = true;
                if !is_invalid_or_blank {
                    if let Some(matcher_result) = app.get_current_regex_matcher(widget_id) {
                        if let Ok(matcher) = matcher_result {
                            if is_searching_with_pid {
                                result = matcher.is_match(&process.pid.to_string());
                            } else {
                                result = matcher.is_match(&process.name);
                            }
                        }
                    }
                }

                if result {
                    return Some(ConvertedProcessData {
                        pid: process.pid,
                        name: process.name.clone(),
                        cpu_usage: process.cpu_usage_percent,
                        mem_usage: process.mem_usage_percent,
                        group_pids: vec![process.pid],
                    });
                }

                None
            })
            .collect::<Vec<_>>()
    };

    // Quick fix for tab updating the table headers
    if let Some(proc_widget_state) = app.proc_state.widget_states.get_mut(&widget_id) {
        if let data_harvester::processes::ProcessSorting::PID =
            proc_widget_state.process_sorting_type
        {
            if proc_widget_state.is_grouped {
                proc_widget_state.process_sorting_type =
                    data_harvester::processes::ProcessSorting::CPU; // Go back to default, negate PID for group
                proc_widget_state.process_sorting_reverse = true;
            }
        }

        let mut resulting_processes = filtered_process_data;
        sort_process_data(&mut resulting_processes, proc_widget_state);

        // FIXME: We may want to also check if the current cursor position is valid

        app.canvas_data
            .finalized_process_data
            .insert(widget_id, resulting_processes);
    }
}

fn sort_process_data(
    to_sort_vec: &mut Vec<ConvertedProcessData>, proc_widget_state: &app::ProcWidgetState,
) {
    to_sort_vec.sort_by(|a, b| utils::gen_util::get_ordering(&a.name, &b.name, false));

    match proc_widget_state.process_sorting_type {
        ProcessSorting::CPU => {
            to_sort_vec.sort_by(|a, b| {
                utils::gen_util::get_ordering(
                    a.cpu_usage,
                    b.cpu_usage,
                    proc_widget_state.process_sorting_reverse,
                )
            });
        }
        ProcessSorting::MEM => {
            to_sort_vec.sort_by(|a, b| {
                utils::gen_util::get_ordering(
                    a.mem_usage,
                    b.mem_usage,
                    proc_widget_state.process_sorting_reverse,
                )
            });
        }
        ProcessSorting::NAME => {
            // Don't repeat if false...
            if proc_widget_state.process_sorting_reverse {
                to_sort_vec.sort_by(|a, b| {
                    utils::gen_util::get_ordering(
                        &a.name,
                        &b.name,
                        proc_widget_state.process_sorting_reverse,
                    )
                })
            }
        }
        ProcessSorting::PID => {
            if !proc_widget_state.is_grouped {
                to_sort_vec.sort_by(|a, b| {
                    utils::gen_util::get_ordering(
                        a.pid,
                        b.pid,
                        proc_widget_state.process_sorting_reverse,
                    )
                });
            }
        }
    }
}

fn create_input_thread(
    tx: std::sync::mpsc::Sender<
        BottomEvent<crossterm::event::KeyEvent, crossterm::event::MouseEvent>,
    >,
) {
    thread::spawn(move || loop {
        if poll(Duration::from_millis(20)).is_ok() {
            let mut mouse_timer = Instant::now();
            let mut keyboard_timer = Instant::now();

            loop {
                if poll(Duration::from_millis(20)).is_ok() {
                    if let Ok(event) = read() {
                        if let Event::Key(key) = event {
                            if Instant::now().duration_since(keyboard_timer).as_millis() >= 20 {
                                if tx.send(BottomEvent::KeyInput(key)).is_err() {
                                    return;
                                }
                                keyboard_timer = Instant::now();
                            }
                        } else if let Event::Mouse(mouse) = event {
                            if Instant::now().duration_since(mouse_timer).as_millis() >= 20 {
                                if tx.send(BottomEvent::MouseInput(mouse)).is_err() {
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
    tx: std::sync::mpsc::Sender<
        BottomEvent<crossterm::event::KeyEvent, crossterm::event::MouseEvent>,
    >,
    rrx: std::sync::mpsc::Receiver<ResetEvent>, use_current_cpu_total: bool,
    update_rate_in_milliseconds: u64, temp_type: data_harvester::temperature::TemperatureType,
    show_average_cpu: bool,
) {
    thread::spawn(move || {
        let tx = tx.clone();
        let mut data_state = data_harvester::DataState::default();
        data_state.init();
        data_state.set_temperature_type(temp_type);
        data_state.set_use_current_cpu_total(use_current_cpu_total);
        data_state.set_show_average_cpu(show_average_cpu);
        loop {
            if let Ok(message) = rrx.try_recv() {
                match message {
                    ResetEvent::Reset => {
                        data_state.data.first_run_cleanup();
                    }
                }
            }
            futures::executor::block_on(data_state.update_data());
            let event = BottomEvent::Update(Box::from(data_state.data));
            data_state.data = data_harvester::Data::default();
            tx.send(event).unwrap();
            thread::sleep(Duration::from_millis(update_rate_in_milliseconds));
        }
    });
}
