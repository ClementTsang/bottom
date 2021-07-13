#![warn(rust_2018_idioms)]
#[allow(unused_imports)]
#[cfg(feature = "log")]
#[macro_use]
extern crate log;

use bottom::{
    canvas,
    drawing::{paint, View},
    options::*,
    *,
};

use std::{
    boxed::Box,
    io::{stdout, Write},
    panic,
    sync::{
        atomic::{AtomicBool, Ordering},
        mpsc, Arc, Condvar, Mutex,
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
    let matches = clap::get_matches();
    #[cfg(all(feature = "fern", debug_assertions))]
    {
        utils::logging::init_logger(log::LevelFilter::Debug, std::ffi::OsStr::new("debug.log"))?;
    }

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
    )?;

    // Create painter and set colours.
    let mut painter = canvas::Painter::init(
        widget_layout,
        app.app_config_fields.table_gap,
        app.app_config_fields.use_basic_mode,
        &config,
        get_color_scheme(&matches, &config)?,
    )?;

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
    {
        let is_terminated = is_terminated.clone();
        ctrlc::set_handler(move || {
            is_terminated.store(true, Ordering::SeqCst);
        })?;
    }
    let mut first_pass = true;

    // ===== Start of actual thread creation and loop =====

    // Create termination mutex and cvar
    #[allow(clippy::mutex_atomic)]
    let thread_termination_lock = Arc::new(Mutex::new(false));
    let thread_termination_cvar = Arc::new(Condvar::new());

    // Set up input handling
    let (sender, receiver) = mpsc::channel();
    let _input_thread = create_input_thread(sender.clone(), thread_termination_lock.clone());

    // Cleaning loop
    let _cleaning_thread = {
        let lock = thread_termination_lock.clone();
        let cvar = thread_termination_cvar.clone();
        let cleaning_sender = sender.clone();
        const OFFSET_WAIT_TIME: u64 = constants::STALE_MAX_MILLISECONDS + 60000;
        thread::spawn(move || {
            loop {
                let result = cvar.wait_timeout(
                    lock.lock().unwrap(),
                    Duration::from_millis(OFFSET_WAIT_TIME),
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

    // Event loop
    let (collection_thread_ctrl_sender, collection_thread_ctrl_receiver) = mpsc::channel();
    let _collection_thread = create_collection_thread(
        sender,
        collection_thread_ctrl_receiver,
        thread_termination_lock.clone(),
        thread_termination_cvar.clone(),
        &app.app_config_fields,
        app.filters.clone(),
        app.used_widgets.clone(),
    );

    while !is_terminated.load(Ordering::SeqCst) {
        if let Ok(recv) = receiver.recv() {
            match recv {
                BottomEvent::KeyInput(event) => {
                    if handle_key_event_or_break(event, &mut app, &collection_thread_ctrl_sender) {
                        break;
                    }
                    handle_force_redraws(&mut app);
                }
                BottomEvent::MouseInput(event) => {
                    handle_mouse_event(event, &mut app);
                    handle_force_redraws(&mut app);
                }
                BottomEvent::Update(data) => {
                    app.data_collection.eat_data(data);

                    // This thing is required as otherwise, some widgets can't draw correctly w/o
                    // some data (or they need to be re-drawn).
                    if first_pass {
                        first_pass = false;
                        app.is_force_redraw = true;
                    }

                    update_app_data(&mut app);
                }
                BottomEvent::Clean => {
                    app.data_collection
                        .clean_data(constants::STALE_MAX_MILLISECONDS);

                    continue;
                }
                BottomEvent::RequestRedraw => {
                    // FIXME: Add draws back to all required locations!
                }
            }

            // TODO: Create new draw state.
            let mut root = View::new(drawing::Axis::Horizontal).into();
            if paint(&mut terminal, &mut root).is_err() {
                break;
            }
        }
    }

    *thread_termination_lock.lock().unwrap() = true;
    thread_termination_cvar.notify_all(); // Tell all threads to stop, even if they are sleeping.
    cleanup_terminal(&mut terminal)?;

    // ===== End of actual thread creation and loop =====

    Ok(())
}
