#![warn(rust_2018_idioms)]
#[allow(unused_imports)]
#[cfg(feature = "log")]
#[macro_use]
extern crate log;

// TODO: Deny unused imports.

pub mod utils {
    pub mod error;
    pub mod gen_util;
    pub mod logging;
}
pub mod canvas;
pub mod clap;
pub mod constants;
pub mod data_conversion;
pub mod drawing;
pub mod options;
pub mod units;

use std::{
    boxed::Box,
    fs,
    io::{stdout, Write},
    panic::PanicInfo,
    path::PathBuf,
    sync::Arc,
    sync::Condvar,
    sync::Mutex,
    thread,
    time::{Duration, Instant},
};

use crossterm::{
    event::{poll, read, DisableMouseCapture, Event, KeyCode, KeyEvent, KeyModifiers, MouseEvent},
    execute,
    style::Print,
    terminal::{disable_raw_mode, LeaveAlternateScreen},
};

use app::{
    data_harvester::{self, processes::ProcessSorting},
    layout_manager::{UsedWidgets, WidgetDirection},
    AppState,
};
use constants::*;
use data_conversion::*;
use options::*;
use utils::error;

pub mod app;

#[cfg(target_family = "windows")]
pub type Pid = usize;

#[cfg(target_family = "unix")]
pub type Pid = libc::pid_t;

#[derive(Debug)]
pub enum BottomEvent<I, J> {
    KeyInput(I),
    MouseInput(J),
    Update(Box<data_harvester::Data>),
    Clean,
    RequestRedraw,
}

#[derive(Debug)]
pub enum ThreadControlEvent {
    Reset,
    UpdateConfig(Box<app::AppConfigFields>),
    UpdateUsedWidgets(Box<UsedWidgets>),
    UpdateUpdateTime(u64),
}

pub fn handle_mouse_event(event: MouseEvent, app: &mut AppState) {
    match event {
        MouseEvent::ScrollUp(_x, _y, _modifiers) => app.handle_scroll_up(),
        MouseEvent::ScrollDown(_x, _y, _modifiers) => app.handle_scroll_down(),
        MouseEvent::Down(button, x, y, _modifiers) => {
            if !app.app_config_fields.disable_click {
                match button {
                    crossterm::event::MouseButton::Left => {
                        // Trigger left click widget activity
                        app.on_left_mouse_up(x, y);
                    }
                    crossterm::event::MouseButton::Right => {}
                    _ => {}
                }
            }
        }
        _ => {}
    };
}

pub fn handle_key_event_or_break(
    event: KeyEvent, app: &mut AppState, reset_sender: &std::sync::mpsc::Sender<ThreadControlEvent>,
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
            KeyCode::F(5) => app.toggle_tree_mode(),
            KeyCode::F(6) => app.toggle_sort(),
            KeyCode::F(9) => app.start_killing_process(),
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
                    if reset_sender.send(ThreadControlEvent::Reset).is_ok() {
                        app.reset();
                    }
                }
                KeyCode::Char('a') => app.skip_cursor_beginning(),
                KeyCode::Char('e') => app.skip_cursor_end(),
                KeyCode::Char('u') => app.clear_search(),
                KeyCode::Char('w') => app.clear_previous_word(),
                KeyCode::Char('h') => app.on_backspace(),
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

pub fn read_config(config_location: Option<&str>) -> error::Result<Option<PathBuf>> {
    let config_path = if let Some(conf_loc) = config_location {
        Some(PathBuf::from(conf_loc))
    } else if cfg!(target_os = "windows") {
        if let Some(home_path) = dirs::config_dir() {
            let mut path = home_path;
            path.push(DEFAULT_CONFIG_FILE_PATH);
            Some(path)
        } else {
            None
        }
    } else if let Some(home_path) = dirs::home_dir() {
        let mut path = home_path;
        path.push(".config/");
        path.push(DEFAULT_CONFIG_FILE_PATH);
        if path.exists() {
            // If it already exists, use the old one.
            Some(path)
        } else {
            // If it does not, use the new one!
            if let Some(config_path) = dirs::config_dir() {
                let mut path = config_path;
                path.push(DEFAULT_CONFIG_FILE_PATH);
                Some(path)
            } else {
                None
            }
        }
    } else {
        None
    };

    Ok(config_path)
}

pub fn create_or_get_config(config_path: &Option<PathBuf>) -> error::Result<Config> {
    if let Some(path) = config_path {
        if let Ok(config_string) = fs::read_to_string(path) {
            // We found a config file!
            Ok(toml::from_str(config_string.as_str())?)
        } else {
            // Config file DNE...
            if let Some(parent_path) = path.parent() {
                fs::create_dir_all(parent_path)?;
            }
            // fs::File::create(path)?.write_all(CONFIG_TOP_HEAD.as_bytes())?;
            fs::File::create(path)?.write_all(CONFIG_TEXT.as_bytes())?;
            Ok(Config::default())
        }
    } else {
        // Don't write, the config path was somehow None...
        Ok(Config::default())
    }
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

    // if is_debug {
    //     let mut tmp_dir = std::env::temp_dir();
    //     tmp_dir.push("bottom_debug.log");
    //     println!("Your debug file is located at {:?}", tmp_dir.as_os_str());
    // }

    Ok(())
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

pub fn handle_force_redraws(app: &mut AppState) {
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
        convert_cpu_data_points(
            &app.data_collection,
            &mut app.canvas_data.cpu_data,
            app.is_frozen,
        );
        app.canvas_data.load_avg_data = app.data_collection.load_avg_harvest;
        app.cpu_state.force_update = None;
    }

    // FIXME: [OPT] Prefer reassignment over new vectors?
    if app.mem_state.force_update.is_some() {
        app.canvas_data.mem_data = convert_mem_data_points(&app.data_collection, app.is_frozen);
        app.canvas_data.swap_data = convert_swap_data_points(&app.data_collection, app.is_frozen);
        app.mem_state.force_update = None;
    }

    if app.net_state.force_update.is_some() {
        let (rx, tx) = get_rx_tx_data_points(
            &app.data_collection,
            app.is_frozen,
            &app.app_config_fields.network_scale_type,
            &app.app_config_fields.network_unit_type,
            app.app_config_fields.network_use_binary_prefix,
        );
        app.canvas_data.network_data_rx = rx;
        app.canvas_data.network_data_tx = tx;
        app.net_state.force_update = None;
    }
}

pub fn update_app_data(app: &mut AppState) {
    if !app.is_frozen {
        // Convert all data into tui-compliant components

        // Network
        if app.used_widgets.use_net {
            let network_data = convert_network_data_points(
                &app.data_collection,
                false,
                app.app_config_fields.use_basic_mode
                    || app.app_config_fields.use_old_network_legend,
                &app.app_config_fields.network_scale_type,
                &app.app_config_fields.network_unit_type,
                app.app_config_fields.network_use_binary_prefix,
            );
            app.canvas_data.network_data_rx = network_data.rx;
            app.canvas_data.network_data_tx = network_data.tx;
            app.canvas_data.rx_display = network_data.rx_display;
            app.canvas_data.tx_display = network_data.tx_display;
            if let Some(total_rx_display) = network_data.total_rx_display {
                app.canvas_data.total_rx_display = total_rx_display;
            }
            if let Some(total_tx_display) = network_data.total_tx_display {
                app.canvas_data.total_tx_display = total_tx_display;
            }
        }

        // Disk
        if app.used_widgets.use_disk {
            app.canvas_data.disk_data = convert_disk_row(&app.data_collection);
        }

        // Temperatures
        if app.used_widgets.use_temp {
            app.canvas_data.temp_sensor_data = convert_temp_row(&app);
        }

        // Memory
        if app.used_widgets.use_mem {
            app.canvas_data.mem_data = convert_mem_data_points(&app.data_collection, false);
            app.canvas_data.swap_data = convert_swap_data_points(&app.data_collection, false);
            let (memory_labels, swap_labels) = convert_mem_labels(&app.data_collection);

            app.canvas_data.mem_labels = memory_labels;
            app.canvas_data.swap_labels = swap_labels;
        }

        if app.used_widgets.use_cpu {
            // CPU
            convert_cpu_data_points(&app.data_collection, &mut app.canvas_data.cpu_data, false);
            app.canvas_data.load_avg_data = app.data_collection.load_avg_harvest;
        }

        // Processes
        if app.used_widgets.use_proc {
            update_all_process_lists(app);
        }

        // Battery
        if app.used_widgets.use_battery {
            app.canvas_data.battery_data = convert_battery_harvest(&app.data_collection);
        }
    }
}

#[allow(clippy::needless_collect)]
pub fn update_all_process_lists(app: &mut AppState) {
    // According to clippy, I can avoid a collect... but if I follow it,
    // I end up conflicting with the borrow checker since app is used within the closure... hm.
    if !app.is_frozen {
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
}

fn update_final_process_list(app: &mut AppState, widget_id: u64) {
    let process_states = app
        .proc_state
        .widget_states
        .get(&widget_id)
        .map(|process_state| {
            (
                process_state
                    .process_search_state
                    .search_state
                    .is_invalid_or_blank_search(),
                process_state.is_using_command,
                process_state.is_grouped,
                process_state.is_tree_mode,
            )
        });

    if let Some((is_invalid_or_blank, is_using_command, is_grouped, is_tree)) = process_states {
        if !app.is_frozen {
            convert_process_data(
                &app.data_collection,
                &mut app.canvas_data.single_process_data,
                #[cfg(target_family = "unix")]
                &mut app.user_table,
            );
        }
        let process_filter = app.get_process_filter(widget_id);
        let filtered_process_data: Vec<ConvertedProcessData> = if is_tree {
            app.canvas_data
                .single_process_data
                .iter()
                .map(|(_pid, process)| {
                    let mut process_clone = process.clone();
                    if !is_invalid_or_blank {
                        if let Some(process_filter) = process_filter {
                            process_clone.is_disabled_entry =
                                !process_filter.check(&process_clone, is_using_command);
                        }
                    }
                    process_clone
                })
                .collect::<Vec<_>>()
        } else {
            app.canvas_data
                .single_process_data
                .iter()
                .filter_map(|(_pid, process)| {
                    if !is_invalid_or_blank {
                        if let Some(process_filter) = process_filter {
                            if process_filter.check(&process, is_using_command) {
                                Some(process)
                            } else {
                                None
                            }
                        } else {
                            Some(process)
                        }
                    } else {
                        Some(process)
                    }
                })
                .cloned()
                .collect::<Vec<_>>()
        };

        if let Some(proc_widget_state) = app.proc_state.get_mut_widget_state(widget_id) {
            let mut finalized_process_data = if is_tree {
                tree_process_data(
                    &filtered_process_data,
                    is_using_command,
                    &proc_widget_state.process_sorting_type,
                    proc_widget_state.is_process_sort_descending,
                )
            } else if is_grouped {
                group_process_data(&filtered_process_data, is_using_command)
            } else {
                filtered_process_data
            };

            // Note tree mode is sorted well before this, as it's special.
            if !is_tree {
                sort_process_data(&mut finalized_process_data, proc_widget_state);
            }

            if proc_widget_state.scroll_state.current_scroll_position
                >= finalized_process_data.len()
            {
                proc_widget_state.scroll_state.current_scroll_position =
                    finalized_process_data.len().saturating_sub(1);
                proc_widget_state.scroll_state.previous_scroll_position = 0;
                proc_widget_state.scroll_state.scroll_direction = app::ScrollDirection::Down;
            }

            app.canvas_data.stringified_process_data_map.insert(
                widget_id,
                stringify_process_data(&proc_widget_state, &finalized_process_data),
            );
            app.canvas_data
                .finalized_process_data_map
                .insert(widget_id, finalized_process_data);
        }
    }
}

fn sort_process_data(
    to_sort_vec: &mut Vec<ConvertedProcessData>, proc_widget_state: &app::ProcWidgetState,
) {
    to_sort_vec.sort_by_cached_key(|c| c.name.to_lowercase());

    match &proc_widget_state.process_sorting_type {
        ProcessSorting::CpuPercent => {
            to_sort_vec.sort_by(|a, b| {
                utils::gen_util::get_ordering(
                    a.cpu_percent_usage,
                    b.cpu_percent_usage,
                    proc_widget_state.is_process_sort_descending,
                )
            });
        }
        ProcessSorting::Mem => {
            to_sort_vec.sort_by(|a, b| {
                utils::gen_util::get_ordering(
                    a.mem_usage_bytes,
                    b.mem_usage_bytes,
                    proc_widget_state.is_process_sort_descending,
                )
            });
        }
        ProcessSorting::MemPercent => {
            to_sort_vec.sort_by(|a, b| {
                utils::gen_util::get_ordering(
                    a.mem_percent_usage,
                    b.mem_percent_usage,
                    proc_widget_state.is_process_sort_descending,
                )
            });
        }
        ProcessSorting::ProcessName => {
            // Don't repeat if false... it sorts by name by default anyways.
            if proc_widget_state.is_process_sort_descending {
                to_sort_vec.sort_by_cached_key(|c| c.name.to_lowercase());
                if proc_widget_state.is_process_sort_descending {
                    to_sort_vec.reverse();
                }
            }
        }
        ProcessSorting::Command => {
            to_sort_vec.sort_by_cached_key(|c| c.command.to_lowercase());
            if proc_widget_state.is_process_sort_descending {
                to_sort_vec.reverse();
            }
        }
        ProcessSorting::Pid => {
            if !proc_widget_state.is_grouped {
                to_sort_vec.sort_by(|a, b| {
                    utils::gen_util::get_ordering(
                        a.pid,
                        b.pid,
                        proc_widget_state.is_process_sort_descending,
                    )
                });
            }
        }
        ProcessSorting::ReadPerSecond => {
            to_sort_vec.sort_by(|a, b| {
                utils::gen_util::get_ordering(
                    a.rps_f64,
                    b.rps_f64,
                    proc_widget_state.is_process_sort_descending,
                )
            });
        }
        ProcessSorting::WritePerSecond => {
            to_sort_vec.sort_by(|a, b| {
                utils::gen_util::get_ordering(
                    a.wps_f64,
                    b.wps_f64,
                    proc_widget_state.is_process_sort_descending,
                )
            });
        }
        ProcessSorting::TotalRead => {
            to_sort_vec.sort_by(|a, b| {
                utils::gen_util::get_ordering(
                    a.tr_f64,
                    b.tr_f64,
                    proc_widget_state.is_process_sort_descending,
                )
            });
        }
        ProcessSorting::TotalWrite => {
            to_sort_vec.sort_by(|a, b| {
                utils::gen_util::get_ordering(
                    a.tw_f64,
                    b.tw_f64,
                    proc_widget_state.is_process_sort_descending,
                )
            });
        }
        ProcessSorting::State => {
            to_sort_vec.sort_by_cached_key(|c| c.process_state.to_lowercase());
            if proc_widget_state.is_process_sort_descending {
                to_sort_vec.reverse();
            }
        }
        ProcessSorting::User => to_sort_vec.sort_by(|a, b| match (&a.user, &b.user) {
            (Some(user_a), Some(user_b)) => utils::gen_util::get_ordering(
                user_a.to_lowercase(),
                user_b.to_lowercase(),
                proc_widget_state.is_process_sort_descending,
            ),
            (Some(_), None) => std::cmp::Ordering::Less,
            (None, Some(_)) => std::cmp::Ordering::Greater,
            (None, None) => std::cmp::Ordering::Less,
        }),
        ProcessSorting::Count => {
            if proc_widget_state.is_grouped {
                to_sort_vec.sort_by(|a, b| {
                    utils::gen_util::get_ordering(
                        a.group_pids.len(),
                        b.group_pids.len(),
                        proc_widget_state.is_process_sort_descending,
                    )
                });
            }
        }
    }
}

pub fn create_input_thread(
    sender: std::sync::mpsc::Sender<
        BottomEvent<crossterm::event::KeyEvent, crossterm::event::MouseEvent>,
    >,
    termination_ctrl_lock: Arc<Mutex<bool>>,
) -> std::thread::JoinHandle<()> {
    thread::spawn(move || {
        let mut mouse_timer = Instant::now();
        let mut keyboard_timer = Instant::now();

        loop {
            if let Ok(is_terminated) = termination_ctrl_lock.try_lock() {
                // We don't block.
                if *is_terminated {
                    drop(is_terminated);
                    break;
                }
            }
            if let Ok(poll) = poll(Duration::from_millis(20)) {
                if poll {
                    if let Ok(event) = read() {
                        match event {
                            Event::Key(key) => {
                                if Instant::now().duration_since(keyboard_timer).as_millis() >= 20 {
                                    if sender.send(BottomEvent::KeyInput(key)).is_err() {
                                        break;
                                    }
                                    keyboard_timer = Instant::now();
                                }
                            }
                            Event::Mouse(mouse) => {
                                if Instant::now().duration_since(mouse_timer).as_millis() >= 20 {
                                    if sender.send(BottomEvent::MouseInput(mouse)).is_err() {
                                        break;
                                    }
                                    mouse_timer = Instant::now();
                                }
                            }
                            Event::Resize(_, _) => {
                                // if sender.send(BottomEvent::RequestRedraw).is_err() {
                                //     break;
                                // }
                            }
                        }
                    }
                }
            }
        }
    })
}

pub fn create_collection_thread(
    sender: std::sync::mpsc::Sender<
        BottomEvent<crossterm::event::KeyEvent, crossterm::event::MouseEvent>,
    >,
    control_receiver: std::sync::mpsc::Receiver<ThreadControlEvent>,
    termination_ctrl_lock: Arc<Mutex<bool>>, termination_ctrl_cvar: Arc<Condvar>,
    app_config_fields: &app::AppConfigFields, filters: app::DataFilters,
    used_widget_set: UsedWidgets,
) -> std::thread::JoinHandle<()> {
    let temp_type = app_config_fields.temperature_type.clone();
    let use_current_cpu_total = app_config_fields.use_current_cpu_total;
    let show_average_cpu = app_config_fields.show_average_cpu;
    let update_rate_in_milliseconds = app_config_fields.update_rate_in_milliseconds;

    thread::spawn(move || {
        let mut data_state = data_harvester::DataCollector::new(filters);

        data_state.set_collected_data(used_widget_set);
        data_state.set_temperature_type(temp_type);
        data_state.set_use_current_cpu_total(use_current_cpu_total);
        data_state.set_show_average_cpu(show_average_cpu);

        data_state.init();

        loop {
            // Check once at the very top...
            if let Ok(is_terminated) = termination_ctrl_lock.try_lock() {
                // We don't block here.
                if *is_terminated {
                    drop(is_terminated);
                    break;
                }
            }

            let mut update_time = update_rate_in_milliseconds;
            if let Ok(message) = control_receiver.try_recv() {
                // trace!("Received message in collection thread: {:?}", message);
                match message {
                    ThreadControlEvent::Reset => {
                        data_state.data.cleanup();
                    }
                    ThreadControlEvent::UpdateConfig(app_config_fields) => {
                        data_state.set_temperature_type(app_config_fields.temperature_type.clone());
                        data_state
                            .set_use_current_cpu_total(app_config_fields.use_current_cpu_total);
                        data_state.set_show_average_cpu(app_config_fields.show_average_cpu);
                    }
                    ThreadControlEvent::UpdateUsedWidgets(used_widget_set) => {
                        data_state.set_collected_data(*used_widget_set);
                    }
                    ThreadControlEvent::UpdateUpdateTime(new_time) => {
                        update_time = new_time;
                    }
                }
            }
            futures::executor::block_on(data_state.update_data());

            // Yet another check to bail if needed...
            if let Ok(is_terminated) = termination_ctrl_lock.try_lock() {
                // We don't block here.
                if *is_terminated {
                    drop(is_terminated);
                    break;
                }
            }

            let event = BottomEvent::Update(Box::from(data_state.data));
            data_state.data = data_harvester::Data::default();
            if sender.send(event).is_err() {
                break;
            }

            if let Ok((is_terminated, _wait_timeout_result)) = termination_ctrl_cvar.wait_timeout(
                termination_ctrl_lock.lock().unwrap(),
                Duration::from_millis(update_time),
            ) {
                if *is_terminated {
                    drop(is_terminated);
                    break;
                }
            }
        }
    })
}
