//! A customizable cross-platform graphical process/system monitor for the terminal.
//! Supports Linux, macOS, and Windows. Inspired by gtop, gotop, and htop.
//!
//! **Note:** The following documentation is primarily intended for people to refer to for development purposes rather
//! than the actual usage of the application. If you are instead looking for documentation regarding the *usage* of
//! bottom, refer to [here](https://clementtsang.github.io/bottom/stable/).

pub mod app;
pub mod utils {
    pub mod data_prefixes;
    pub mod data_units;
    pub mod error;
    pub mod general;
    pub mod logging;
    pub mod strings;
}
pub mod canvas;
pub mod constants;
pub mod data_collection;
pub mod data_conversion;
pub mod event;
pub mod options;
pub mod widgets;

use std::{
    boxed::Box,
    fs,
    io::{stderr, stdout, Write},
    panic::PanicInfo,
    path::{Path, PathBuf},
    sync::{
        mpsc::{Receiver, Sender},
        Arc, Condvar, Mutex,
    },
    thread::{self, JoinHandle},
    time::{Duration, Instant},
};

use app::{
    frozen_state::FrozenState, layout_manager::UsedWidgets, App, AppConfigFields, DataFilters,
};
use constants::*;
use crossterm::{
    event::{
        poll, read, DisableBracketedPaste, DisableMouseCapture, Event, KeyEventKind, MouseEventKind,
    },
    execute,
    style::Print,
    terminal::{disable_raw_mode, LeaveAlternateScreen},
};
use data_conversion::*;
use event::{BottomEvent, CollectionThreadEvent};
pub use options::args;
use options::ConfigV1;
use utils::error;
#[allow(unused_imports)]
pub use utils::logging::*;

#[cfg(target_family = "windows")]
pub type Pid = usize;

#[cfg(target_family = "unix")]
pub type Pid = libc::pid_t;

pub fn get_config_path(override_config_path: Option<&Path>) -> Option<PathBuf> {
    if let Some(conf_loc) = override_config_path {
        Some(conf_loc.to_path_buf())
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
    }
}

pub fn get_or_create_config(override_config_path: Option<&Path>) -> error::Result<ConfigV1> {
    let config_path = get_config_path(override_config_path);

    if let Some(path) = &config_path {
        if let Ok(config_string) = fs::read_to_string(path) {
            Ok(toml_edit::de::from_str(config_string.as_str())?)
        } else {
            if let Some(parent_path) = path.parent() {
                fs::create_dir_all(parent_path)?;
            }

            fs::File::create(path)?.write_all(CONFIG_TEXT.as_bytes())?;
            Ok(ConfigV1::default())
        }
    } else {
        // If we somehow don't have any config path, then just assume the default config but don't write to any file.
        Ok(ConfigV1::default())
    }
}

pub fn try_drawing(
    terminal: &mut tui::terminal::Terminal<tui::backend::CrosstermBackend<std::io::Stdout>>,
    app: &mut App, painter: &mut canvas::Painter,
) -> error::Result<()> {
    if let Err(err) = painter.draw_data(terminal, app) {
        cleanup_terminal(terminal)?;
        Err(err)
    } else {
        Ok(())
    }
}

pub fn cleanup_terminal(
    terminal: &mut tui::terminal::Terminal<tui::backend::CrosstermBackend<std::io::Stdout>>,
) -> error::Result<()> {
    disable_raw_mode()?;
    execute!(
        terminal.backend_mut(),
        DisableBracketedPaste,
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
/// Originally based on [spotify-tui's implementation](https://github.com/Rigellute/spotify-tui/blob/master/src/main.rs).
pub fn panic_hook(panic_info: &PanicInfo<'_>) {
    let mut stdout = stdout();

    let msg = match panic_info.payload().downcast_ref::<&'static str>() {
        Some(s) => *s,
        None => match panic_info.payload().downcast_ref::<String>() {
            Some(s) => &s[..],
            None => "Box<Any>",
        },
    };

    let backtrace = format!("{:?}", backtrace::Backtrace::new());

    let _ = disable_raw_mode();
    let _ = execute!(
        stdout,
        DisableBracketedPaste,
        DisableMouseCapture,
        LeaveAlternateScreen
    );

    // Print stack trace. Must be done after!
    if let Some(panic_info) = panic_info.location() {
        let _ = execute!(
            stdout,
            Print(format!(
                "thread '<unnamed>' panicked at '{msg}', {panic_info}\n\r{backtrace}",
            )),
        );
    }
}

pub fn update_data(app: &mut App) {
    let data_source = match &app.frozen_state {
        FrozenState::NotFrozen => &app.data_collection,
        FrozenState::Frozen(data) => data,
    };

    for proc in app.states.proc_state.widget_states.values_mut() {
        if proc.force_update_data {
            proc.set_table_data(data_source);
            proc.force_update_data = false;
        }
    }

    // FIXME: Make this CPU force update less terrible.
    if app.states.cpu_state.force_update.is_some() {
        app.converted_data.convert_cpu_data(data_source);
        app.converted_data.load_avg_data = data_source.load_avg_harvest;

        app.states.cpu_state.force_update = None;
    }

    // FIXME: This is a bit of a temp hack to move data over.
    {
        let data = &app.converted_data.cpu_data;
        for cpu in app.states.cpu_state.widget_states.values_mut() {
            cpu.update_table(data);
        }
    }
    {
        let data = &app.converted_data.temp_data;
        for temp in app.states.temp_state.widget_states.values_mut() {
            if temp.force_update_data {
                temp.set_table_data(data);
                temp.force_update_data = false;
            }
        }
    }
    {
        let data = &app.converted_data.disk_data;
        for disk in app.states.disk_state.widget_states.values_mut() {
            if disk.force_update_data {
                disk.set_table_data(data);
                disk.force_update_data = false;
            }
        }
    }

    // TODO: [OPT] Prefer reassignment over new vectors?
    if app.states.mem_state.force_update.is_some() {
        app.converted_data.mem_data = convert_mem_data_points(data_source);
        #[cfg(not(target_os = "windows"))]
        {
            app.converted_data.cache_data = convert_cache_data_points(data_source);
        }
        app.converted_data.swap_data = convert_swap_data_points(data_source);
        #[cfg(feature = "zfs")]
        {
            app.converted_data.arc_data = convert_arc_data_points(data_source);
        }

        #[cfg(feature = "gpu")]
        {
            app.converted_data.gpu_data = convert_gpu_data(data_source);
        }
        app.states.mem_state.force_update = None;
    }

    if app.states.net_state.force_update.is_some() {
        let (rx, tx) = get_network_points(
            data_source,
            &app.app_config_fields.network_scale_type,
            &app.app_config_fields.network_unit_type,
            app.app_config_fields.network_use_binary_prefix,
        );
        app.converted_data.network_data_rx = rx;
        app.converted_data.network_data_tx = tx;
        app.states.net_state.force_update = None;
    }
}

pub fn create_input_thread(
    sender: Sender<BottomEvent>, termination_ctrl_lock: Arc<Mutex<bool>>,
) -> JoinHandle<()> {
    thread::spawn(move || {
        let mut mouse_timer = Instant::now();

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
                            Event::Resize(_, _) => {
                                // TODO: Might want to debounce this in the future, or take into account the actual resize
                                // values. Maybe we want to keep the current implementation in case the resize event might
                                // not fire... not sure.

                                if sender.send(BottomEvent::Resize).is_err() {
                                    break;
                                }
                            }
                            Event::Paste(paste) => {
                                if sender.send(BottomEvent::PasteEvent(paste)).is_err() {
                                    break;
                                }
                            }
                            Event::Key(key) if key.kind == KeyEventKind::Press => {
                                // For now, we only care about key down events. This may change in the future.
                                if sender.send(BottomEvent::KeyInput(key)).is_err() {
                                    break;
                                }
                            }
                            Event::Mouse(mouse) => match mouse.kind {
                                MouseEventKind::Moved | MouseEventKind::Drag(..) => {}
                                MouseEventKind::ScrollDown | MouseEventKind::ScrollUp => {
                                    if Instant::now().duration_since(mouse_timer).as_millis() >= 20
                                    {
                                        if sender.send(BottomEvent::MouseInput(mouse)).is_err() {
                                            break;
                                        }
                                        mouse_timer = Instant::now();
                                    }
                                }
                                _ => {
                                    if sender.send(BottomEvent::MouseInput(mouse)).is_err() {
                                        break;
                                    }
                                }
                            },
                            Event::Key(_) => {}
                            Event::FocusGained => {}
                            Event::FocusLost => {}
                        }
                    }
                }
            }
        }
    })
}

pub fn create_collection_thread(
    sender: Sender<BottomEvent>, control_receiver: Receiver<CollectionThreadEvent>,
    termination_lock: Arc<Mutex<bool>>, termination_cvar: Arc<Condvar>,
    app_config_fields: &AppConfigFields, filters: DataFilters, used_widget_set: UsedWidgets,
) -> JoinHandle<()> {
    let temp_type = app_config_fields.temperature_type;
    let use_current_cpu_total = app_config_fields.use_current_cpu_total;
    let unnormalized_cpu = app_config_fields.unnormalized_cpu;
    let show_average_cpu = app_config_fields.show_average_cpu;
    let update_time = app_config_fields.update_rate;

    thread::spawn(move || {
        let mut data_state = data_collection::DataCollector::new(filters);

        data_state.set_data_collection(used_widget_set);
        data_state.set_temperature_type(temp_type);
        data_state.set_use_current_cpu_total(use_current_cpu_total);
        data_state.set_unnormalized_cpu(unnormalized_cpu);
        data_state.set_show_average_cpu(show_average_cpu);

        data_state.init();

        loop {
            // Check once at the very top... don't block though.
            if let Ok(is_terminated) = termination_lock.try_lock() {
                if *is_terminated {
                    drop(is_terminated);
                    break;
                }
            }

            if let Ok(message) = control_receiver.try_recv() {
                // trace!("Received message in collection thread: {message:?}");
                match message {
                    CollectionThreadEvent::Reset => {
                        data_state.data.cleanup();
                    }
                }
            }

            data_state.update_data();

            // Yet another check to bail if needed... do not block!
            if let Ok(is_terminated) = termination_lock.try_lock() {
                if *is_terminated {
                    drop(is_terminated);
                    break;
                }
            }

            let event = BottomEvent::Update(Box::from(data_state.data));
            data_state.data = data_collection::Data::default();
            if sender.send(event).is_err() {
                break;
            }

            // This is actually used as a "sleep" that can be interrupted by another thread.
            if let Ok((is_terminated, _)) = termination_cvar.wait_timeout(
                termination_lock.lock().unwrap(),
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
