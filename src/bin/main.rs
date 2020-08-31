#![warn(rust_2018_idioms)]
#[allow(unused_imports)]
#[macro_use]
extern crate log;

use bottom::{canvas, constants::*, data_conversion::*, options::*, utils::error, *};

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

use crossterm::{
    event::EnableMouseCapture,
    execute,
    terminal::{enable_raw_mode, EnterAlternateScreen},
};
use tui::{backend::CrosstermBackend, Terminal};

fn main() -> error::Result<()> {
    #[cfg(debug_assertions)]
    {
        utils::logging::init_logger()?;
    }
    let matches = clap::get_matches();

    let config: Config = create_config(matches.value_of("CONFIG_LOCATION"))?;

    // Get widget layout separately
    let (widget_layout, default_widget_id, default_widget_type_option) =
        get_widget_layout(&matches, &config)?;

    // Create "app" struct, which will control most of the program and store settings/state
    let mut app = build_app(
        &matches,
        &config,
        &widget_layout,
        default_widget_id,
        &default_widget_type_option,
    )?;

    // Create painter and set colours.
    let mut painter = canvas::Painter::init(
        widget_layout,
        app.app_config_fields.table_gap,
        app.app_config_fields.use_basic_mode,
    );
    generate_config_colours(&config, &mut painter)?;
    painter.colours.generate_remaining_cpu_colours();
    painter.complete_painter_init();

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
    create_event_thread(
        sender,
        reset_receiver,
        app.app_config_fields.use_current_cpu_total,
        app.app_config_fields.update_rate_in_milliseconds,
        app.app_config_fields.temperature_type.clone(),
        app.app_config_fields.show_average_cpu,
        app.used_widgets.clone(),
    );

    // Set up up tui and crossterm
    let mut stdout_val = stdout();
    execute!(stdout_val, EnterAlternateScreen, EnableMouseCapture)?;
    enable_raw_mode()?;

    let mut terminal = Terminal::new(CrosstermBackend::new(stdout_val))?;
    terminal.hide_cursor()?;

    // Set panic hook
    panic::set_hook(Box::new(|info| panic_hook(info)));

    // Set termination hook
    let is_terminated = Arc::new(AtomicBool::new(false));
    let ist_clone = is_terminated.clone();
    ctrlc::set_handler(move || {
        ist_clone.store(true, Ordering::SeqCst);
        termination_hook();
    })
    .unwrap();

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
                            app.canvas_data.disk_data = convert_disk_row(&app.data_collection);
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
