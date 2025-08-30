//! A customizable cross-platform graphical process/system monitor for the
//! terminal. Supports Linux, macOS, and Windows. Inspired by gtop, gotop, and
//! htop.
//!
//! **Note:** The following documentation is primarily intended for people to
//! refer to for development purposes rather than the actual usage of the
//! application. If you are instead looking for documentation regarding the
//! *usage* of bottom, refer to [here](https://clementtsang.github.io/bottom/stable/).

pub(crate) mod app;
mod utils {
    pub(crate) mod cancellation_token;
    pub(crate) mod conversion;
    pub(crate) mod data_units;
    pub(crate) mod general;
    pub(crate) mod logging;
    pub(crate) mod process_killer;
    pub(crate) mod strings;
}
pub(crate) mod canvas;
pub(crate) mod collection;
pub(crate) mod constants;
pub(crate) mod event;
pub mod options;
pub mod widgets;

use std::{
    boxed::Box,
    io::{Write, stderr, stdout},
    panic::{self, PanicHookInfo},
    sync::{
        Arc,
        mpsc::{self, Receiver, Sender},
    },
    thread::{self, JoinHandle},
    time::{Duration, Instant},
};

use app::{App, AppConfigFields, DataFilters, layout_manager::UsedWidgets};
use crossterm::{
    cursor::{Hide, Show},
    event::{
        DisableBracketedPaste, DisableMouseCapture, EnableBracketedPaste, EnableMouseCapture,
        Event, KeyEventKind, MouseEventKind, poll, read,
    },
    execute,
    terminal::{EnterAlternateScreen, LeaveAlternateScreen, disable_raw_mode, enable_raw_mode},
};
use event::{BottomEvent, CollectionThreadEvent, handle_key_event_or_break, handle_mouse_event};
use options::{args, get_or_create_config, init_app};
use tui::{Terminal, backend::CrosstermBackend};
#[allow(unused_imports, reason = "this is needed if logging is enabled")]
use utils::logging::*;
use utils::{cancellation_token::CancellationToken, conversion::*};

use crate::collection::Data;

// Used for heap allocation debugging purposes.
// #[global_allocator]
// static ALLOC: dhat::Alloc = dhat::Alloc;

/// Try drawing. If not, clean up the terminal and return an error.
fn try_drawing(
    terminal: &mut Terminal<CrosstermBackend<std::io::Stdout>>, app: &mut App,
    painter: &mut canvas::Painter,
) -> anyhow::Result<()> {
    if let Err(err) = painter.draw_data(terminal, app) {
        cleanup_terminal(terminal)?;
        Err(err.into())
    } else {
        Ok(())
    }
}

/// Clean up the terminal before returning it to the user.
fn cleanup_terminal(
    terminal: &mut Terminal<CrosstermBackend<std::io::Stdout>>,
) -> anyhow::Result<()> {
    disable_raw_mode()?;

    execute!(
        terminal.backend_mut(),
        DisableMouseCapture,
        DisableBracketedPaste,
        LeaveAlternateScreen,
        Show,
    )?;
    terminal.show_cursor()?;

    Ok(())
}

/// Check and report to the user if the current environment is not a terminal.
fn check_if_terminal() {
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

/// This manually resets stdout back to normal state.
pub fn reset_stdout() {
    let mut stdout = stdout();
    let _ = disable_raw_mode();
    let _ = execute!(
        stdout,
        DisableMouseCapture,
        DisableBracketedPaste,
        LeaveAlternateScreen,
        Show,
    );
}

/// A panic hook to properly restore the terminal in the case of a panic.
/// Originally based on [spotify-tui's implementation](https://github.com/Rigellute/spotify-tui/blob/master/src/main.rs).
fn panic_hook(panic_info: &PanicHookInfo<'_>) {
    let msg = match panic_info.payload().downcast_ref::<&'static str>() {
        Some(s) => *s,
        None => match panic_info.payload().downcast_ref::<String>() {
            Some(s) => &s[..],
            None => "Box<Any>",
        },
    };

    let backtrace = format!("{:?}", backtrace::Backtrace::new());

    reset_stdout();

    // Print stack trace. Must be done after!
    if let Some(panic_info) = panic_info.location() {
        println!("thread '<unnamed>' panicked at '{msg}', {panic_info}\n\r{backtrace}")
    }

    // TODO: Might be cleaner in the future to use a cancellation token, but that causes some fun issues with
    // lifetimes; for now if it panics then shut down the main program entirely ASAP.
    std::process::exit(1);
}

/// Create a thread to poll for user inputs and forward them to the main thread.
fn create_input_thread(
    sender: Sender<BottomEvent>, cancellation_token: Arc<CancellationToken>,
    app_config_fields: &AppConfigFields,
) -> JoinHandle<()> {
    let keys_disabled = app_config_fields.disable_keys;

    thread::spawn(move || {
        let mut mouse_timer = Instant::now();

        loop {
            // We don't block.
            if let Some(is_terminated) = cancellation_token.try_check() {
                if is_terminated {
                    break;
                }
            }

            if let Ok(poll) = poll(Duration::from_millis(20)) {
                if poll {
                    if let Ok(event) = read() {
                        match event {
                            Event::Resize(_, _) => {
                                // TODO: Might want to debounce this in the future, or take into
                                // account the actual resize values.
                                // Maybe we want to keep the current implementation in case the
                                // resize event might not fire...
                                // not sure.

                                if sender.send(BottomEvent::Resize).is_err() {
                                    break;
                                }
                            }
                            Event::Paste(paste) => {
                                if sender.send(BottomEvent::PasteEvent(paste)).is_err() {
                                    break;
                                }
                            }
                            Event::Key(key)
                                if !keys_disabled && key.kind == KeyEventKind::Press =>
                            {
                                // For now, we only care about key down events. This may change in
                                // the future.
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

/// Create a thread to handle data collection.
fn create_collection_thread(
    sender: Sender<BottomEvent>, control_receiver: Receiver<CollectionThreadEvent>,
    cancellation_token: Arc<CancellationToken>, app_config_fields: &AppConfigFields,
    filters: DataFilters, used_widget_set: UsedWidgets,
) -> JoinHandle<()> {
    let use_current_cpu_total = app_config_fields.use_current_cpu_total;
    let unnormalized_cpu = app_config_fields.unnormalized_cpu;
    let show_average_cpu = app_config_fields.show_average_cpu;
    let update_sleep = app_config_fields.update_rate;
    let get_process_threads = app_config_fields.get_process_threads;

    thread::spawn(move || {
        let mut data_collector = collection::DataCollector::new(filters);

        data_collector.set_collection(used_widget_set);
        data_collector.set_use_current_cpu_total(use_current_cpu_total);
        data_collector.set_unnormalized_cpu(unnormalized_cpu);
        data_collector.set_show_average_cpu(show_average_cpu);
        data_collector.set_get_process_threads(get_process_threads);

        data_collector.update_data();
        data_collector.data = Data::default();

        // Tiny sleep I guess? To go between the first update above and the first update in the loop.
        std::thread::sleep(Duration::from_millis(5));

        loop {
            // Check once at the very top... don't block though.
            if let Some(is_terminated) = cancellation_token.try_check() {
                if is_terminated {
                    break;
                }
            }

            if let Ok(message) = control_receiver.try_recv() {
                // trace!("Received message in collection thread: {message:?}");
                match message {
                    CollectionThreadEvent::Reset => {
                        data_collector.data.cleanup();
                    }
                }
            }

            data_collector.update_data();

            // Yet another check to bail if needed... do not block!
            if let Some(is_terminated) = cancellation_token.try_check() {
                if is_terminated {
                    break;
                }
            }

            let event = BottomEvent::Update(Box::from(data_collector.data));
            data_collector.data = Data::default();

            if sender.send(event).is_err() {
                break;
            }

            // Sleep while allowing for interruptions...
            if cancellation_token.sleep_with_cancellation(Duration::from_millis(update_sleep)) {
                break;
            }
        }
    })
}

/// Main code to call to start bottom.
#[inline]
pub fn start_bottom(enable_error_hook: &mut bool) -> anyhow::Result<()> {
    // let _profiler = dhat::Profiler::new_heap();

    let args = args::get_args();

    #[cfg(feature = "logging")]
    {
        if let Err(err) = init_logger(
            log::LevelFilter::Debug,
            Some(std::ffi::OsStr::new("debug.log")),
        ) {
            println!("Issue initializing logger: {err}");
        }
    }

    // Read from config file.
    let config = get_or_create_config(args.general.config_location.as_deref())?;

    // Create the "app" and initialize a bunch of stuff.
    let (mut app, widget_layout, styling) = init_app(args, config)?;

    // Create painter and set colours.
    let mut painter = canvas::Painter::init(widget_layout, styling)?;

    // Check if the current environment is in a terminal.
    check_if_terminal();

    let cancellation_token = Arc::new(CancellationToken::default());
    let (sender, receiver) = mpsc::channel();

    // Set up the event loop thread; we set this up early to speed up
    // first-time-to-data.
    let (collection_thread_ctrl_sender, collection_thread_ctrl_receiver) = mpsc::channel();
    let _collection_thread = create_collection_thread(
        sender.clone(),
        collection_thread_ctrl_receiver,
        cancellation_token.clone(),
        &app.app_config_fields,
        app.filters.clone(),
        app.used_widgets,
    );

    // Set up the input handling loop thread.
    let _input_thread = create_input_thread(
        sender.clone(),
        cancellation_token.clone(),
        &app.app_config_fields,
    );

    // Set up the cleaning loop thread.
    let _cleaning_thread = {
        let cancellation_token = cancellation_token.clone();
        let cleaning_sender = sender.clone();
        let offset_wait = Duration::from_millis(app.app_config_fields.retention_ms + 60000);
        thread::spawn(move || {
            loop {
                if cancellation_token.sleep_with_cancellation(offset_wait) {
                    break;
                }

                if cleaning_sender.send(BottomEvent::Clean).is_err() {
                    break;
                }
            }
        })
    };

    // Set up tui and crossterm
    *enable_error_hook = true;

    let mut stdout_val = stdout();
    execute!(stdout_val, Hide, EnterAlternateScreen, EnableBracketedPaste)?;
    if app.app_config_fields.disable_click {
        execute!(stdout_val, DisableMouseCapture)?;
    } else {
        execute!(stdout_val, EnableMouseCapture)?;
    }
    enable_raw_mode()?;

    let mut terminal = Terminal::new(CrosstermBackend::new(stdout_val))?;
    terminal.clear()?;
    terminal.hide_cursor()?;

    #[cfg(target_os = "freebsd")]
    let _stderr_fd = {
        // A really ugly band-aid to suppress stderr warnings on FreeBSD due to sysinfo.
        // For more information, see https://github.com/ClementTsang/bottom/issues/798.
        use std::fs::OpenOptions;

        use filedescriptor::{FileDescriptor, StdioDescriptor};

        let path = OpenOptions::new().write(true).open("/dev/null")?;
        FileDescriptor::redirect_stdio(&path, StdioDescriptor::Stderr)?
    };

    // Set panic hook
    panic::set_hook(Box::new(panic_hook));

    // Set termination hook
    ctrlc::set_handler(move || {
        // TODO: Consider using signal-hook (https://github.com/vorner/signal-hook) to handle
        // more types of signals?
        let _ = sender.send(BottomEvent::Terminate);
    })?;

    let mut first_run = true;

    // Draw once first to initialize the canvas, so it doesn't feel like it's
    // frozen.
    try_drawing(&mut terminal, &mut app, &mut painter)?;

    loop {
        if let Ok(recv) = receiver.recv() {
            match recv {
                BottomEvent::Terminate => break,
                BottomEvent::Resize => {
                    try_drawing(&mut terminal, &mut app, &mut painter)?;
                }
                BottomEvent::KeyInput(event) => {
                    if handle_key_event_or_break(event, &mut app, &collection_thread_ctrl_sender) {
                        break;
                    }
                    app.update_data();
                    try_drawing(&mut terminal, &mut app, &mut painter)?;
                }
                BottomEvent::MouseInput(event) => {
                    handle_mouse_event(event, &mut app);
                    app.update_data();
                    try_drawing(&mut terminal, &mut app, &mut painter)?;
                }
                BottomEvent::PasteEvent(paste) => {
                    app.handle_paste(paste);
                    app.update_data();
                    try_drawing(&mut terminal, &mut app, &mut painter)?;
                }
                BottomEvent::Update(data) => {
                    app.data_store.eat_data(data, &app.app_config_fields);

                    // This thing is required as otherwise, some widgets can't draw correctly w/o
                    // some data (or they need to be re-drawn).
                    if first_run {
                        first_run = false;
                        app.is_force_redraw = true;
                    }

                    if !app.data_store.is_frozen() {
                        // Convert all data into data for the displayed widgets.

                        if app.used_widgets.use_disk {
                            for disk in app.states.disk_state.widget_states.values_mut() {
                                disk.force_data_update();
                            }
                        }

                        if app.used_widgets.use_temp {
                            for temp in app.states.temp_state.widget_states.values_mut() {
                                temp.force_data_update();
                            }
                        }

                        if app.used_widgets.use_proc {
                            for proc in app.states.proc_state.widget_states.values_mut() {
                                proc.force_data_update();
                            }
                        }

                        if app.used_widgets.use_cpu {
                            for cpu in app.states.cpu_state.widget_states.values_mut() {
                                cpu.force_data_update();
                            }
                        }

                        app.update_data();
                        try_drawing(&mut terminal, &mut app, &mut painter)?;
                    }
                }
                BottomEvent::Clean => {
                    app.data_store
                        .clean_data(Duration::from_millis(app.app_config_fields.retention_ms));
                }
            }
        }
    }

    // I think doing it in this order is safe...
    // TODO: maybe move the cancellation token to the ctrl-c handler?
    cancellation_token.cancel();
    cleanup_terminal(&mut terminal)?;

    Ok(())
}
