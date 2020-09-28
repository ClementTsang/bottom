#![warn(rust_2018_idioms)]
#[allow(unused_imports)]
#[macro_use]
extern crate log;

use bottom::{canvas, constants::*, data_conversion::*, options::*, *};

use std::{
    boxed::Box,
    io::{stdout, Write},
    panic,
    sync::{
        atomic::{AtomicBool, Ordering},
        mpsc, Arc,
    },
    thread,
    time::Duration,
};

use anyhow::{Context, Result};
use crossterm::{
    event::EnableMouseCapture,
    execute,
    terminal::{enable_raw_mode, EnterAlternateScreen},
};
use tui::{backend::CrosstermBackend, Terminal};

fn main() -> Result<()> {
    #[cfg(debug_assertions)]
    {
        utils::logging::init_logger()?;
    }
    let matches = clap::get_matches();

    let config_path = read_config(matches.value_of("config_location"))
        .context("Unable to access the given config file location.")?;
    let mut config: Config = create_or_get_config(&config_path)
        .context("Unable to properly parse or create the config file.")?;

    // Get widget layout separately
    let (widget_layout, default_widget_id, default_widget_type_option) =
        get_widget_layout(&matches, &config)
            .context("Found an issue while trying to build the widget layout.")?;

    // Create "app" struct, which will control most of the program and store settings/state
    let mut app = build_app(
        &matches,
        &mut config,
        &widget_layout,
        default_widget_id,
        &default_widget_type_option,
        config_path,
    )?;

    // Create painter and set colours.
    let mut painter = canvas::Painter::init(
        widget_layout,
        app.app_config_fields.table_gap,
        app.app_config_fields.use_basic_mode,
        &config,
    )?;

    // Set up input handling
    let (sender, receiver) = mpsc::channel();
    create_input_thread(sender.clone());

    // Cleaning loop
    {
        let cleaning_sender = sender.clone();
        thread::spawn(move || loop {
            thread::sleep(Duration::from_millis(
                constants::STALE_MAX_MILLISECONDS + 5000,
            ));
            if cleaning_sender.send(BottomEvent::Clean).is_err() {
                break;
            }
        });
    }

    // Event loop
    let (reset_sender, reset_receiver) = mpsc::channel();
    create_collection_thread(
        sender,
        reset_receiver,
        &app.app_config_fields,
        app.used_widgets.clone(),
    );

    // Set up up tui and crossterm
    let mut stdout_val = stdout();
    execute!(stdout_val, EnterAlternateScreen, EnableMouseCapture)?;
    enable_raw_mode()?;

    let mut terminal = Terminal::new(CrosstermBackend::new(stdout_val))?;
    terminal.clear()?;
    terminal.hide_cursor()?;

    // Set panic hook
    panic::set_hook(Box::new(|info| panic_hook(info)));

    // Set termination hook
    let is_terminated = Arc::new(AtomicBool::new(false));
    let ist_clone = is_terminated.clone();
    ctrlc::set_handler(move || {
        ist_clone.store(true, Ordering::SeqCst);
        termination_hook();
    })?;
    let mut first_run = true;

    while !is_terminated.load(Ordering::SeqCst) {
        if let Ok(recv) = receiver.recv_timeout(Duration::from_millis(TICK_RATE_IN_MILLISECONDS)) {
            match recv {
                BottomEvent::KeyInput(event) => {
                    if handle_key_event_or_break(event, &mut app, &reset_sender) {
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

                    // This thing is required as otherwise, some widgets can't draw correctly w/o
                    // some data (or they need to be re-drawn).
                    if first_run {
                        first_run = false;
                        app.is_force_redraw = true;
                    }

                    if !app.is_frozen {
                        // Convert all data into tui-compliant components

                        // Network
                        if app.used_widgets.use_net {
                            let network_data = convert_network_data_points(
                                &app.data_collection,
                                false,
                                app.app_config_fields.use_basic_mode
                                    || app.app_config_fields.use_old_network_legend,
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
                            app.canvas_data.disk_data =
                                convert_disk_row(&app.data_collection, &app.filters.disk_filter);
                        }

                        // Temperatures
                        if app.used_widgets.use_temp {
                            app.canvas_data.temp_sensor_data = convert_temp_row(&app);
                        }

                        // Memory
                        if app.used_widgets.use_mem {
                            app.canvas_data.mem_data =
                                convert_mem_data_points(&app.data_collection, false);
                            app.canvas_data.swap_data =
                                convert_swap_data_points(&app.data_collection, false);
                            let memory_and_swap_labels = convert_mem_labels(&app.data_collection);
                            app.canvas_data.mem_label_percent = memory_and_swap_labels.0;
                            app.canvas_data.mem_label_frac = memory_and_swap_labels.1;
                            app.canvas_data.swap_label_percent = memory_and_swap_labels.2;
                            app.canvas_data.swap_label_frac = memory_and_swap_labels.3;
                        }

                        if app.used_widgets.use_cpu {
                            // CPU
                            app.canvas_data.cpu_data =
                                convert_cpu_data_points(&app.data_collection, false);
                        }

                        // Processes
                        if app.used_widgets.use_proc {
                            update_all_process_lists(&mut app);
                        }

                        // Battery
                        if app.used_widgets.use_battery {
                            app.canvas_data.battery_data =
                                convert_battery_harvest(&app.data_collection);
                        }
                    }
                }
                BottomEvent::Clean => {
                    app.data_collection
                        .clean_data(constants::STALE_MAX_MILLISECONDS);
                }
            }
        }

        // TODO: [OPT] Should not draw if no change (ie: scroll max)
        try_drawing(&mut terminal, &mut app, &mut painter)?;
    }

    cleanup_terminal(&mut terminal)?;
    Ok(())
}
