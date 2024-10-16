//! A customizable cross-platform graphical process/system monitor for the
//! terminal. Supports Linux, macOS, and Windows. Inspired by gtop, gotop, and
//! htop.
//!
//! **Note:** The following documentation is primarily intended for people to
//! refer to for development purposes rather than the actual usage of the
//! application. If you are instead looking for documentation regarding the
//! *usage* of bottom, refer to [here](https://clementtsang.github.io/bottom/stable/).

pub mod app;
pub mod utils {
    pub mod cancellation_token;
    pub mod data_prefixes;
    pub mod data_units;
    pub mod general;
    pub mod logging;
    pub mod strings;
}
pub mod canvas;
pub mod constants;
pub mod data_collection;
pub mod data_conversion;
pub mod event;
pub mod new_data_collection;
pub mod options;
pub mod widgets;

use std::{
    boxed::Box,
    io::{stderr, stdout, Write},
    panic::{self, PanicInfo},
    sync::{
        mpsc::{self, Receiver, Sender},
        Arc,
    },
    thread::{self, JoinHandle},
    time::{Duration, Instant},
};

use app::{layout_manager::UsedWidgets, App, AppConfigFields, DataFilters};
use crossterm::{
    event::{
        poll, read, DisableBracketedPaste, DisableMouseCapture, EnableBracketedPaste,
        EnableMouseCapture, Event, KeyEventKind, MouseEventKind,
    },
    execute,
    style::Print,
    terminal::{disable_raw_mode, enable_raw_mode, EnterAlternateScreen, LeaveAlternateScreen},
};
use data_conversion::*;
use event::{handle_key_event_or_break, handle_mouse_event, BottomEvent, CollectionThreadEvent};
use options::{args, get_or_create_config, init_app};
use tui::{backend::CrosstermBackend, Terminal};
use utils::cancellation_token::CancellationToken;
#[allow(unused_imports)]
use utils::logging::*;

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
        DisableBracketedPaste,
        DisableMouseCapture,
        LeaveAlternateScreen
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

/// A panic hook to properly restore the terminal in the case of a panic.
/// Originally based on [spotify-tui's implementation](https://github.com/Rigellute/spotify-tui/blob/master/src/main.rs).
fn panic_hook(panic_info: &PanicInfo<'_>) {
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

    // TODO: Might be cleaner in the future to use a cancellation token, but that causes some fun issues with
    // lifetimes; for now if it panics then shut down the main program entirely ASAP.
    std::process::exit(1);
}

/// Create a thread to poll for user inputs and forward them to the main thread.
fn create_input_thread(
    sender: Sender<BottomEvent>, cancellation_token: Arc<CancellationToken>,
) -> JoinHandle<()> {
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
                            Event::Key(key) if key.kind == KeyEventKind::Press => {
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
            if let Some(is_terminated) = cancellation_token.try_check() {
                if is_terminated {
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
            if let Some(is_terminated) = cancellation_token.try_check() {
                if is_terminated {
                    break;
                }
            }

            let event = BottomEvent::Update(Box::from(data_state.data));
            data_state.data = data_collection::Data::default();
            if sender.send(event).is_err() {
                break;
            }

            // Sleep while allowing for interruptions...
            if cancellation_token.sleep_with_cancellation(Duration::from_millis(update_time)) {
                break;
            }
        }
    })
}

#[cfg(feature = "generate_schema")]
fn generate_schema() -> anyhow::Result<()> {
    let mut schema = schemars::schema_for!(crate::options::config::Config);
    {
        use itertools::Itertools;
        use strum::VariantArray;

        let proc_columns = schema.definitions.get_mut("ProcColumn").unwrap();
        match proc_columns {
            schemars::schema::Schema::Object(proc_columns) => {
                let enums = proc_columns.enum_values.as_mut().unwrap();
                *enums = options::config::process::ProcColumn::VARIANTS
                    .iter()
                    .flat_map(|var| var.get_schema_names())
                    .map(|v| serde_json::Value::String(v.to_string()))
                    .dedup()
                    .collect();
            }
            _ => anyhow::bail!("missing proc columns definition"),
        }
    }

    let metadata = schema.schema.metadata.as_mut().unwrap();
    metadata.id = Some(
        "https://github.com/ClementTsang/bottom/blob/main/schema/nightly/bottom.json".to_string(),
    );
    metadata.description =
        Some("https://clementtsang.github.io/bottom/nightly/configuration/config-file".to_string());
    println!("{}", serde_json::to_string_pretty(&schema).unwrap());

    Ok(())
}

fn main() -> anyhow::Result<()> {
    // let _profiler = dhat::Profiler::new_heap();

    let args = args::get_args();

    #[cfg(feature = "generate_schema")]
    if args.other.generate_schema {
        return generate_schema();
    }

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
    let _input_thread = create_input_thread(sender.clone(), cancellation_token.clone());

    // Set up the cleaning loop thread.
    let _cleaning_thread = {
        let cancellation_token = cancellation_token.clone();
        let cleaning_sender = sender.clone();
        let offset_wait_time = app.app_config_fields.retention_ms + 60000;
        thread::spawn(move || loop {
            if cancellation_token.sleep_with_cancellation(Duration::from_millis(offset_wait_time)) {
                break;
            }

            if cleaning_sender.send(BottomEvent::Clean).is_err() {
                break;
            }
        })
    };

    // Set up tui and crossterm
    let mut stdout_val = stdout();
    execute!(
        stdout_val,
        EnterAlternateScreen,
        EnableMouseCapture,
        EnableBracketedPaste
    )?;
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
        let _ = sender.send(BottomEvent::Terminate);
    })?;

    let mut first_run = true;

    // Draw once first to initialize the canvas, so it doesn't feel like it's
    // frozen.
    try_drawing(&mut terminal, &mut app, &mut painter)?;

    loop {
        if let Ok(recv) = receiver.recv() {
            match recv {
                BottomEvent::Terminate => {
                    break;
                }
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
                    app.data_collection.eat_data(data);

                    // This thing is required as otherwise, some widgets can't draw correctly w/o
                    // some data (or they need to be re-drawn).
                    if first_run {
                        first_run = false;
                        app.is_force_redraw = true;
                    }

                    if !app.frozen_state.is_frozen() {
                        // Convert all data into data for the displayed widgets.

                        if app.used_widgets.use_net {
                            let network_data = convert_network_points(
                                &app.data_collection,
                                app.app_config_fields.use_basic_mode
                                    || app.app_config_fields.use_old_network_legend,
                                &app.app_config_fields.network_scale_type,
                                &app.app_config_fields.network_unit_type,
                                app.app_config_fields.network_use_binary_prefix,
                            );
                            app.converted_data.network_data_rx = network_data.rx;
                            app.converted_data.network_data_tx = network_data.tx;
                            app.converted_data.rx_display = network_data.rx_display;
                            app.converted_data.tx_display = network_data.tx_display;
                            if let Some(total_rx_display) = network_data.total_rx_display {
                                app.converted_data.total_rx_display = total_rx_display;
                            }
                            if let Some(total_tx_display) = network_data.total_tx_display {
                                app.converted_data.total_tx_display = total_tx_display;
                            }
                        }

                        if app.used_widgets.use_disk {
                            app.converted_data.convert_disk_data(&app.data_collection);

                            for disk in app.states.disk_state.widget_states.values_mut() {
                                disk.force_data_update();
                            }
                        }

                        if app.used_widgets.use_temp {
                            app.converted_data.convert_temp_data(
                                &app.data_collection,
                                app.app_config_fields.temperature_type,
                            );

                            for temp in app.states.temp_state.widget_states.values_mut() {
                                temp.force_data_update();
                            }
                        }

                        if app.used_widgets.use_mem {
                            app.converted_data.mem_data =
                                convert_mem_data_points(&app.data_collection);

                            #[cfg(not(target_os = "windows"))]
                            {
                                app.converted_data.cache_data =
                                    convert_cache_data_points(&app.data_collection);
                            }

                            app.converted_data.swap_data =
                                convert_swap_data_points(&app.data_collection);

                            #[cfg(feature = "zfs")]
                            {
                                app.converted_data.arc_data =
                                    convert_arc_data_points(&app.data_collection);
                            }

                            #[cfg(feature = "gpu")]
                            {
                                app.converted_data.gpu_data =
                                    convert_gpu_data(&app.data_collection);
                            }

                            app.converted_data.mem_labels =
                                convert_mem_label(&app.data_collection.memory_harvest);

                            app.converted_data.swap_labels =
                                convert_mem_label(&app.data_collection.swap_harvest);

                            #[cfg(not(target_os = "windows"))]
                            {
                                app.converted_data.cache_labels =
                                    convert_mem_label(&app.data_collection.cache_harvest);
                            }

                            #[cfg(feature = "zfs")]
                            {
                                app.converted_data.arc_labels =
                                    convert_mem_label(&app.data_collection.arc_harvest);
                            }
                        }

                        if app.used_widgets.use_cpu {
                            app.converted_data.convert_cpu_data(&app.data_collection);
                            app.converted_data.load_avg_data = app.data_collection.load_avg_harvest;
                        }

                        if app.used_widgets.use_proc {
                            for proc in app.states.proc_state.widget_states.values_mut() {
                                proc.force_data_update();
                            }
                        }

                        #[cfg(feature = "battery")]
                        {
                            if app.used_widgets.use_battery {
                                app.converted_data.battery_data =
                                    convert_battery_harvest(&app.data_collection);
                            }
                        }

                        app.update_data();
                        try_drawing(&mut terminal, &mut app, &mut painter)?;
                    }
                }
                BottomEvent::Clean => {
                    app.data_collection
                        .clean_data(app.app_config_fields.retention_ms);
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
