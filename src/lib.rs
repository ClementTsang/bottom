#![warn(rust_2018_idioms)]
#[allow(unused_imports)]
#[cfg(feature = "log")]
#[macro_use]
extern crate log;

// TODO: [Style] Deny unused imports.

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
    event::{poll, read, DisableMouseCapture, Event, KeyEvent, MouseEvent, MouseEventKind},
    execute,
    style::Print,
    terminal::{disable_raw_mode, LeaveAlternateScreen},
};

use app::{data_harvester, AppState, UsedWidgets};
use constants::*;
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
pub mod constants;
pub mod data_conversion;
pub mod options;
pub(crate) mod units;

// FIXME: Use newtype pattern for PID
#[cfg(target_family = "windows")]
pub type Pid = usize;

#[cfg(target_family = "unix")]
pub type Pid = libc::pid_t;

#[derive(Debug)]
pub enum BottomEvent {
    KeyInput(KeyEvent),
    MouseInput(MouseEvent),
    Update(Box<data_harvester::Data>),
    Resize { width: u16, height: u16 },
    Clean,
}

#[derive(Debug)]
pub enum ThreadControlEvent {
    Reset,
    UpdateConfig(Box<app::AppConfigFields>),
    UpdateUsedWidgets(Box<UsedWidgets>),
    UpdateUpdateTime(u64),
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
    app: &mut AppState, painter: &mut canvas::Painter,
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

/// Based on https://github.com/Rigellute/spotify-tui/blob/master/src/main.rs
#[allow(clippy::mutex_atomic)]
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

pub fn create_input_thread(
    sender: std::sync::mpsc::Sender<BottomEvent>, termination_ctrl_lock: Arc<Mutex<bool>>,
) -> std::thread::JoinHandle<()> {
    thread::spawn(move || {
        // TODO: [Optimization, Input] Maybe experiment with removing these timers. Look into using buffers instead?
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
                            Event::Key(event) => {
                                if Instant::now().duration_since(keyboard_timer).as_millis() >= 20 {
                                    if sender.send(BottomEvent::KeyInput(event)).is_err() {
                                        break;
                                    }
                                    keyboard_timer = Instant::now();
                                }
                            }
                            Event::Mouse(event) => match &event.kind {
                                MouseEventKind::Drag(_) => {}
                                MouseEventKind::Moved => {}
                                _ => {
                                    if Instant::now().duration_since(mouse_timer).as_millis() >= 20
                                    {
                                        if sender.send(BottomEvent::MouseInput(event)).is_err() {
                                            break;
                                        }
                                        mouse_timer = Instant::now();
                                    }
                                }
                            },
                            Event::Resize(width, height) => {
                                if sender.send(BottomEvent::Resize { width, height }).is_err() {
                                    break;
                                }
                            }
                        }
                    }
                }
            }
        }
    })
}

pub fn create_collection_thread(
    sender: std::sync::mpsc::Sender<BottomEvent>,
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
