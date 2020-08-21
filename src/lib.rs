#![warn(rust_2018_idioms)]
#[allow(unused_imports)]
#[macro_use]
extern crate log;

use std::{
    boxed::Box,
    io::{stdout, Write},
    panic::PanicInfo,
    thread,
    time::{Duration, Instant},
};

use clap::*;

use crossterm::{
    event::{poll, read, DisableMouseCapture, Event, KeyCode, KeyEvent, KeyModifiers, MouseEvent},
    execute,
    style::Print,
    terminal::{disable_raw_mode, LeaveAlternateScreen},
};

use app::{
    data_harvester::{self, processes::ProcessSorting},
    layout_manager::{UsedWidgets, WidgetDirection},
    App,
};
use constants::*;
use data_conversion::*;
use options::*;
use utils::error;

pub mod app;

pub mod utils {
    pub mod error;
    pub mod gen_util;
    pub mod logging;
}

pub mod canvas;
pub mod constants;
pub mod data_conversion;
pub mod options;

pub enum BottomEvent<I, J> {
    KeyInput(I),
    MouseInput(J),
    Update(Box<data_harvester::Data>),
    Clean,
}

pub enum ResetEvent {
    Reset,
}

pub fn get_matches() -> clap::ArgMatches<'static> {
    clap_app!(app =>
		(name: crate_name!())
		(version: crate_version!())
		(author: crate_authors!())
		(about: crate_description!())
		(@arg HIDE_AVG_CPU: -a --hide_avg_cpu "Hides the average CPU usage.")
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
        (@arg DEFAULT_TIME_VALUE: -t --default_time_value +takes_value "Default time value for graphs in milliseconds; minimum is 30s, defaults to 60s.")
        (@arg TIME_DELTA: -d --time_delta +takes_value "The amount changed upon zooming in/out in milliseconds; minimum is 1s, defaults to 15s.")
        (@arg HIDE_TIME: --hide_time "Completely hide the time scaling")
        (@arg AUTOHIDE_TIME: --autohide_time "Automatically hide the time scaling in graphs after being shown for a brief moment when zoomed in/out.  If time is disabled via --hide_time then this will have no effect.")
        (@arg DEFAULT_WIDGET_TYPE: --default_widget_type +takes_value "The default widget type to select by default.")
        (@arg DEFAULT_WIDGET_COUNT: --default_widget_count +takes_value "Which number of the selected widget type to select, from left to right, top to bottom.  Defaults to 1.")
        (@arg USE_OLD_NETWORK_LEGEND: --use_old_network_legend "Use the older (pre-0.4) network widget legend.")
        (@arg HIDE_TABLE_GAP: --hide_table_gap "Hides the spacing between the table headers and entries.")
        (@arg BATTERY: --battery "Shows the battery widget in default or basic mode.  No effect on custom layouts.")
	)
        .get_matches()
}

pub fn handle_mouse_event(event: MouseEvent, app: &mut App) {
    match event {
        MouseEvent::ScrollUp(_x, _y, _modifiers) => app.handle_scroll_up(),
        MouseEvent::ScrollDown(_x, _y, _modifiers) => app.handle_scroll_down(),
        _ => {}
    };
}

pub fn handle_key_event_or_break(
    event: KeyEvent, app: &mut App, reset_sender: &std::sync::mpsc::Sender<ResetEvent>,
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
            KeyCode::F(1) => app.toggle_ignore_case(),
            KeyCode::F(2) => app.toggle_search_whole_word(),
            KeyCode::F(3) => app.toggle_search_regex(),
            KeyCode::F(6) => app.toggle_sort(),
            _ => {}
        }
    } else {
        // Otherwise, track the modifier as well...
        if let KeyModifiers::ALT = event.modifiers {
            match event.code {
                KeyCode::Char('c') | KeyCode::Char('C') => app.toggle_ignore_case(),
                KeyCode::Char('w') | KeyCode::Char('W') => app.toggle_search_whole_word(),
                KeyCode::Char('r') | KeyCode::Char('R') => app.toggle_search_regex(),
                KeyCode::Char('h') => app.on_left_key(),
                KeyCode::Char('l') => app.on_right_key(),
                _ => {}
            }
        } else if let KeyModifiers::CONTROL = event.modifiers {
            if event.code == KeyCode::Char('c') {
                return true;
            }

            match event.code {
                KeyCode::Char('f') => app.on_slash(),
                KeyCode::Left => app.move_widget_selection(&WidgetDirection::Left),
                KeyCode::Right => app.move_widget_selection(&WidgetDirection::Right),
                KeyCode::Up => app.move_widget_selection(&WidgetDirection::Up),
                KeyCode::Down => app.move_widget_selection(&WidgetDirection::Down),
                KeyCode::Char('r') => {
                    if reset_sender.send(ResetEvent::Reset).is_ok() {
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
                KeyCode::Left => app.move_widget_selection(&WidgetDirection::Left),
                KeyCode::Right => app.move_widget_selection(&WidgetDirection::Right),
                KeyCode::Up => app.move_widget_selection(&WidgetDirection::Up),
                KeyCode::Down => app.move_widget_selection(&WidgetDirection::Down),
                KeyCode::Char(caught_char) => app.on_char_key(caught_char),
                _ => {}
            }
        }
    }

    false
}

pub fn create_config(flag_config_location: Option<&str>) -> error::Result<Config> {
    use std::{ffi::OsString, fs};
    let config_path = if let Some(conf_loc) = flag_config_location {
        Some(OsString::from(conf_loc))
    } else if cfg!(target_os = "windows") {
        if let Some(home_path) = dirs::config_dir() {
            let mut path = home_path;
            path.push(DEFAULT_CONFIG_FILE_PATH);
            Some(path.into_os_string())
        } else {
            None
        }
    } else if let Some(home_path) = dirs::home_dir() {
        let mut path = home_path;
        path.push(".config/");
        path.push(DEFAULT_CONFIG_FILE_PATH);
        if path.exists() {
            // If it already exists, use the old one.
            Some(path.into_os_string())
        } else {
            // If it does not, use the new one!
            if let Some(config_path) = dirs::config_dir() {
                let mut path = config_path;
                path.push(DEFAULT_CONFIG_FILE_PATH);
                Some(path.into_os_string())
            } else {
                None
            }
        }
    } else {
        None
    };

    if let Some(config_path) = config_path {
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
    } else {
        // Don't write otherwise...
        Ok(toml::from_str(DEFAULT_CONFIG_CONTENT)?)
    }
}

pub fn try_drawing(
    terminal: &mut tui::terminal::Terminal<tui::backend::CrosstermBackend<std::io::Stdout>>,
    app: &mut App, painter: &mut canvas::Painter,
) -> error::Result<()> {
    if let Err(err) = painter.draw_data(terminal, app) {
        cleanup_terminal(terminal)?;
        return Err(err);
    }

    Ok(())
}

pub fn generate_config_colours(
    config: &Config, painter: &mut canvas::Painter,
) -> error::Result<()> {
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

        if let Some(all_cpu_color) = &colours.all_cpu_color {
            painter.colours.set_all_cpu_colour(all_cpu_color)?;
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

        // if let Some(rx_total_color) = &colours.rx_total_color {
        //     painter.colours.set_rx_total_colour(rx_total_color)?;
        // }

        // if let Some(tx_total_color) = &colours.tx_total_color {
        //     painter.colours.set_tx_total_colour(tx_total_color)?;
        // }

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

        if let Some(battery_colors) = &colours.battery_colors {
            painter.colours.set_battery_colours(battery_colors)?;
        }
    }

    Ok(())
}

pub fn cleanup_terminal(
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

pub fn termination_hook() {
    let mut stdout = stdout();
    disable_raw_mode().unwrap();
    execute!(stdout, DisableMouseCapture, LeaveAlternateScreen).unwrap();
}

/// Based on https://github.com/Rigellute/spotify-tui/blob/master/src/main.rs
pub fn panic_hook(panic_info: &PanicInfo<'_>) {
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
    execute!(stdout, DisableMouseCapture, LeaveAlternateScreen).unwrap();

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

pub fn handle_force_redraws(app: &mut App) {
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

pub fn update_all_process_lists(app: &mut App) {
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

pub fn update_final_process_list(app: &mut App, widget_id: u64) {
    let is_invalid_or_blank = match app.proc_state.widget_states.get(&widget_id) {
        Some(process_state) => process_state
            .process_search_state
            .search_state
            .is_invalid_or_blank_search(),
        None => false,
    };
    let is_grouped = app.is_grouped(widget_id);

    if let Some(proc_widget_state) = app.proc_state.get_mut_widget_state(widget_id) {
        app.canvas_data.process_data = convert_process_data(
            &app.data_collection,
            if is_grouped {
                ProcessGroupingType::Grouped
            } else {
                ProcessGroupingType::Ungrouped
            },
            if proc_widget_state.is_using_command {
                ProcessNamingType::Path
            } else {
                ProcessNamingType::Name
            },
        );
    }

    let process_filter = app.get_process_filter(widget_id);
    let filtered_process_data: Vec<ConvertedProcessData> = app
        .canvas_data
        .process_data
        .iter()
        .filter(|process| {
            if !is_invalid_or_blank {
                if let Some(process_filter) = process_filter {
                    process_filter.check(&process)
                } else {
                    true
                }
            } else {
                true
            }
        })
        .cloned()
        .collect::<Vec<_>>();

    // Quick fix for tab updating the table headers
    if let Some(proc_widget_state) = app.proc_state.get_mut_widget_state(widget_id) {
        if let data_harvester::processes::ProcessSorting::Pid =
            proc_widget_state.process_sorting_type
        {
            if proc_widget_state.is_grouped {
                proc_widget_state.process_sorting_type =
                    data_harvester::processes::ProcessSorting::CpuPercent; // Go back to default, negate PID for group
                proc_widget_state.process_sorting_reverse = true;
            }
        }

        let mut resulting_processes = filtered_process_data;
        sort_process_data(&mut resulting_processes, proc_widget_state);

        if proc_widget_state.scroll_state.current_scroll_position >= resulting_processes.len() {
            proc_widget_state.scroll_state.current_scroll_position =
                resulting_processes.len().saturating_sub(1);
            proc_widget_state.scroll_state.previous_scroll_position = 0;
            proc_widget_state.scroll_state.scroll_direction = app::ScrollDirection::Down;
        }

        app.canvas_data
            .finalized_process_data_map
            .insert(widget_id, resulting_processes);
    }
}

pub fn sort_process_data(
    to_sort_vec: &mut Vec<ConvertedProcessData>, proc_widget_state: &app::ProcWidgetState,
) {
    to_sort_vec.sort_by(|a, b| {
        utils::gen_util::get_ordering(&a.name.to_lowercase(), &b.name.to_lowercase(), false)
    });

    match proc_widget_state.process_sorting_type {
        ProcessSorting::CpuPercent => {
            to_sort_vec.sort_by(|a, b| {
                utils::gen_util::get_ordering(
                    a.cpu_percent_usage,
                    b.cpu_percent_usage,
                    proc_widget_state.process_sorting_reverse,
                )
            });
        }
        ProcessSorting::Mem => {
            to_sort_vec.sort_by(|a, b| {
                utils::gen_util::get_ordering(
                    a.mem_usage_kb,
                    b.mem_usage_kb,
                    proc_widget_state.process_sorting_reverse,
                )
            });
        }
        ProcessSorting::MemPercent => {
            to_sort_vec.sort_by(|a, b| {
                utils::gen_util::get_ordering(
                    a.mem_percent_usage,
                    b.mem_percent_usage,
                    proc_widget_state.process_sorting_reverse,
                )
            });
        }
        ProcessSorting::ProcessName | ProcessSorting::Command => {
            // Don't repeat if false... it sorts by name by default anyways.
            if proc_widget_state.process_sorting_reverse {
                to_sort_vec.sort_by(|a, b| {
                    utils::gen_util::get_ordering(
                        &a.name.to_lowercase(),
                        &b.name.to_lowercase(),
                        proc_widget_state.process_sorting_reverse,
                    )
                })
            }
        }
        ProcessSorting::Pid => {
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
        ProcessSorting::ReadPerSecond => {
            to_sort_vec.sort_by(|a, b| {
                utils::gen_util::get_ordering(
                    a.rps_f64,
                    b.rps_f64,
                    proc_widget_state.process_sorting_reverse,
                )
            });
        }
        ProcessSorting::WritePerSecond => {
            to_sort_vec.sort_by(|a, b| {
                utils::gen_util::get_ordering(
                    a.wps_f64,
                    b.wps_f64,
                    proc_widget_state.process_sorting_reverse,
                )
            });
        }
        ProcessSorting::TotalRead => {
            to_sort_vec.sort_by(|a, b| {
                utils::gen_util::get_ordering(
                    a.tr_f64,
                    b.tr_f64,
                    proc_widget_state.process_sorting_reverse,
                )
            });
        }
        ProcessSorting::TotalWrite => {
            to_sort_vec.sort_by(|a, b| {
                utils::gen_util::get_ordering(
                    a.tw_f64,
                    b.tw_f64,
                    proc_widget_state.process_sorting_reverse,
                )
            });
        }
        ProcessSorting::State => to_sort_vec.sort_by(|a, b| {
            utils::gen_util::get_ordering(
                &a.process_state.to_lowercase(),
                &b.process_state.to_lowercase(),
                proc_widget_state.process_sorting_reverse,
            )
        }),
    }
}

pub fn create_input_thread(
    sender: std::sync::mpsc::Sender<
        BottomEvent<crossterm::event::KeyEvent, crossterm::event::MouseEvent>,
    >,
) {
    thread::spawn(move || {
        let mut mouse_timer = Instant::now();
        let mut keyboard_timer = Instant::now();

        loop {
            if poll(Duration::from_millis(20)).is_ok() {
                if let Ok(event) = read() {
                    if let Event::Key(key) = event {
                        if Instant::now().duration_since(keyboard_timer).as_millis() >= 20 {
                            if sender.send(BottomEvent::KeyInput(key)).is_err() {
                                break;
                            }
                            keyboard_timer = Instant::now();
                        }
                    } else if let Event::Mouse(mouse) = event {
                        if Instant::now().duration_since(mouse_timer).as_millis() >= 20 {
                            if sender.send(BottomEvent::MouseInput(mouse)).is_err() {
                                break;
                            }
                            mouse_timer = Instant::now();
                        }
                    }
                }
            }
        }
    });
}

pub fn create_event_thread(
    sender: std::sync::mpsc::Sender<
        BottomEvent<crossterm::event::KeyEvent, crossterm::event::MouseEvent>,
    >,
    reset_receiver: std::sync::mpsc::Receiver<ResetEvent>, use_current_cpu_total: bool,
    update_rate_in_milliseconds: u64, temp_type: data_harvester::temperature::TemperatureType,
    show_average_cpu: bool, used_widget_set: UsedWidgets,
) {
    thread::spawn(move || {
        let mut data_state = data_harvester::DataCollector::default();
        data_state.set_collected_data(used_widget_set);
        data_state.set_temperature_type(temp_type);
        data_state.set_use_current_cpu_total(use_current_cpu_total);
        data_state.set_show_average_cpu(show_average_cpu);
        data_state.init();
        loop {
            if let Ok(message) = reset_receiver.try_recv() {
                match message {
                    ResetEvent::Reset => {
                        data_state.data.first_run_cleanup();
                    }
                }
            }
            futures::executor::block_on(data_state.update_data());
            let event = BottomEvent::Update(Box::from(data_state.data));
            data_state.data = data_harvester::Data::default();
            if sender.send(event).is_err() {
                break;
            }
            thread::sleep(Duration::from_millis(update_rate_in_milliseconds));
        }
    });
}
