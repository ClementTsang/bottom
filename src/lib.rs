//! A customizable cross-platform graphical process/system monitor for the terminal.
//! Supports Linux, macOS, and Windows. Inspired by gtop, gotop, and htop.
//!
//! **Note:** The following documentation is primarily intended for people to refer to for development purposes rather
//! than the actual usage of the application. If you are instead looking for documentation regarding the *usage* of
//! bottom, refer to [here](https://clementtsang.github.io/bottom/stable/).

#![warn(rust_2018_idioms)]
#[allow(unused_imports)]
#[cfg(feature = "log")]
#[macro_use]
extern crate log;

// TODO: Deny unused imports.

use std::{
    boxed::Box,
    fs,
    io::{stderr, stdout, Write},
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
    data_harvester,
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
pub mod clap;
pub mod components;
pub mod constants;
pub mod data_conversion;
pub mod options;
pub mod units;

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
}

#[derive(Debug)]
pub enum ThreadControlEvent {
    Reset,
    UpdateConfig(Box<app::AppConfigFields>),
    UpdateUsedWidgets(Box<UsedWidgets>),
    UpdateUpdateTime(u64),
}

pub fn handle_mouse_event(event: MouseEvent, app: &mut App) {
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
    event: KeyEvent, app: &mut App, reset_sender: &std::sync::mpsc::Sender<ThreadControlEvent>,
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
            KeyCode::PageDown => app.on_page_down(),
            KeyCode::PageUp => app.on_page_up(),
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
                KeyCode::Char('u') if app.is_in_search_widget() => app.clear_search(),
                KeyCode::Char('w') => app.clear_previous_word(),
                KeyCode::Char('h') => app.on_backspace(),
                KeyCode::Char('d') => app.scroll_half_page_down(),
                KeyCode::Char('u') => app.scroll_half_page_up(),
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

/// Check and report to the user if the current environment is not a terminal.
pub fn check_if_terminal() {
    use crossterm::tty::IsTty;

    if !stdout().is_tty() {
        eprintln!(
            "Warning: bottom is not being output to a terminal. Things might not work properly."
        );
        eprintln!("If you're stuck, press 'q' or 'Ctrl-c' to quit the program.");
        stderr().flush().unwrap();
        thread::sleep(Duration::from_secs(1));
    }
}

/// A panic hook to properly restore the terminal in the case of a panic.
/// Based on [spotify-tui's implementation](https://github.com/Rigellute/spotify-tui/blob/master/src/main.rs).
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

pub fn update_data(app: &mut App) {
    for proc in app.proc_state.widget_states.values_mut() {
        if proc.force_update_data {
            proc.update_displayed_process_data(&app.data_collection);
            proc.force_update_data = false;
        }
    }

    if app.cpu_state.force_update.is_some() {
        convert_cpu_data_points(&app.data_collection, &mut app.converted_data.cpu_data);
        app.converted_data.load_avg_data = app.data_collection.load_avg_harvest;
        app.cpu_state.force_update = None;
    }

    // TODO: [OPT] Prefer reassignment over new vectors?
    if app.mem_state.force_update.is_some() {
        app.converted_data.mem_data = convert_mem_data_points(&app.data_collection);
        app.converted_data.swap_data = convert_swap_data_points(&app.data_collection);
        #[cfg(feature = "zfs")]
        {
            app.converted_data.arc_data = convert_arc_data_points(&app.data_collection);
        }
        #[cfg(feature = "gpu")]
        {
            app.converted_data.gpu_data = convert_gpu_data(&app.data_collection);
        }
        app.mem_state.force_update = None;
    }

    if app.net_state.force_update.is_some() {
        let (rx, tx) = get_rx_tx_data_points(
            &app.data_collection,
            &app.app_config_fields.network_scale_type,
            &app.app_config_fields.network_unit_type,
            app.app_config_fields.network_use_binary_prefix,
        );
        app.converted_data.network_data_rx = rx;
        app.converted_data.network_data_tx = tx;
        app.net_state.force_update = None;
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

        data_state.set_data_collection(used_widget_set);
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
                        data_state.set_data_collection(*used_widget_set);
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
