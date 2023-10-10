#![deny(rust_2018_idioms)]
#![deny(clippy::todo)]
#![deny(clippy::unimplemented)]
#![deny(clippy::missing_safety_doc)]

use std::{
    boxed::Box,
    io::stdout,
    panic,
    sync::{mpsc, Arc, Condvar, Mutex},
    thread,
    time::Duration,
};

use anyhow::{Context, Result};
use crossterm::{
    event::{EnableBracketedPaste, EnableMouseCapture},
    execute,
    terminal::{enable_raw_mode, EnterAlternateScreen},
};
use tui::{backend::CrosstermBackend, Terminal};

use bottom::{
    canvas::{self, canvas_styling::CanvasStyling},
    data_conversion::*,
    options::*,
    *,
};

// Used for heap allocation debugging purposes.
// #[global_allocator]
// static ALLOC: dhat::Alloc = dhat::Alloc;

fn main() -> Result<()> {
    // let _profiler = dhat::Profiler::new_heap();

    let matches = args::get_matches();

    #[cfg(feature = "logging")]
    {
        if let Err(err) = init_logger(log::LevelFilter::Debug, std::ffi::OsStr::new("debug.log")) {
            println!("Issue initializing logger: {err}");
        }
    }

    // Read from config file.
    let config = {
        let config_path = read_config(matches.get_one::<String>("config_location"))
            .context("Unable to access the given config file location.")?;

        create_or_get_config(&config_path)
            .context("Unable to properly parse or create the config file.")?
    };

    // Get widget layout separately
    let (widget_layout, default_widget_id, default_widget_type_option) =
        get_widget_layout(&matches, &config)
            .context("Found an issue while trying to build the widget layout.")?;

    // FIXME: Should move this into build app or config
    let styling = {
        let colour_scheme = get_color_scheme(&matches, &config)?;
        CanvasStyling::new(colour_scheme, &config)?
    };

    // Create an "app" struct, which will control most of the program and store settings/state
    let mut app = build_app(
        matches,
        config,
        &widget_layout,
        default_widget_id,
        &default_widget_type_option,
        &styling,
    )?;

    // Create painter and set colours.
    let mut painter = canvas::Painter::init(widget_layout, styling)?;

    // Check if the current environment is in a terminal.
    check_if_terminal();

    // Create termination mutex and cvar. We use this setup because we need to sleep at some points in the update
    // thread, but we want to be able to interrupt the "sleep" if a termination occurs.
    let termination_lock = Arc::new(Mutex::new(false));
    let termination_cvar = Arc::new(Condvar::new());

    let (sender, receiver) = mpsc::channel();

    // Set up the event loop thread; we set this up early to speed up first-time-to-data.
    let (collection_thread_ctrl_sender, collection_thread_ctrl_receiver) = mpsc::channel();
    let _collection_thread = create_collection_thread(
        sender.clone(),
        collection_thread_ctrl_receiver,
        termination_lock.clone(),
        termination_cvar.clone(),
        &app.app_config_fields,
        app.filters.clone(),
        app.used_widgets,
    );

    // Set up the input handling loop thread.
    let _input_thread = create_input_thread(sender.clone(), termination_lock.clone());

    // Set up the cleaning loop thread.
    let _cleaning_thread = {
        let lock = termination_lock.clone();
        let cvar = termination_cvar.clone();
        let cleaning_sender = sender.clone();
        let offset_wait_time = app.app_config_fields.retention_ms + 60000;
        thread::spawn(move || {
            loop {
                let result = cvar.wait_timeout(
                    lock.lock().unwrap(),
                    Duration::from_millis(offset_wait_time),
                );
                if let Ok(result) = result {
                    if *(result.0) {
                        break;
                    }
                }
                if cleaning_sender.send(BottomEvent::Clean).is_err() {
                    // debug!("Failed to send cleaning sender...");
                    break;
                }
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
        use filedescriptor::{FileDescriptor, StdioDescriptor};
        use std::fs::OpenOptions;

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

    // Draw once first to initialize the canvas, so it doesn't feel like it's frozen.
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
                    update_data(&mut app);
                    try_drawing(&mut terminal, &mut app, &mut painter)?;
                }
                BottomEvent::MouseInput(event) => {
                    handle_mouse_event(event, &mut app);
                    update_data(&mut app);
                    try_drawing(&mut terminal, &mut app, &mut painter)?;
                }
                BottomEvent::PasteEvent(paste) => {
                    app.handle_paste(paste);
                    update_data(&mut app);
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
                        // Convert all data into tui-compliant components

                        // Network
                        if app.used_widgets.use_net {
                            let network_data = convert_network_data_points(
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

                        // Disk
                        if app.used_widgets.use_disk {
                            app.converted_data.ingest_disk_data(&app.data_collection);

                            for disk in app.states.disk_state.widget_states.values_mut() {
                                disk.force_data_update();
                            }
                        }

                        // Temperatures
                        if app.used_widgets.use_temp {
                            app.converted_data.ingest_temp_data(
                                &app.data_collection,
                                app.app_config_fields.temperature_type,
                            );

                            for temp in app.states.temp_state.widget_states.values_mut() {
                                temp.force_data_update();
                            }
                        }

                        // Memory
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
                                let arc_labels = convert_arc_labels(&app.data_collection);
                                app.converted_data.arc_labels = arc_labels;
                            }
                        }

                        // CPU
                        if app.used_widgets.use_cpu {
                            app.converted_data.ingest_cpu_data(&app.data_collection);
                            app.converted_data.load_avg_data = app.data_collection.load_avg_harvest;
                        }

                        // Processes
                        if app.used_widgets.use_proc {
                            for proc in app.states.proc_state.widget_states.values_mut() {
                                proc.force_data_update();
                            }
                        }

                        // Battery
                        #[cfg(feature = "battery")]
                        {
                            if app.used_widgets.use_battery {
                                app.converted_data.battery_data =
                                    convert_battery_harvest(&app.data_collection);
                            }
                        }

                        update_data(&mut app);
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
    *termination_lock.lock().unwrap() = true;
    termination_cvar.notify_all();
    cleanup_terminal(&mut terminal)?;

    Ok(())
}
