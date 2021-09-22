#![warn(rust_2018_idioms)]
#[allow(unused_imports)]
#[cfg(feature = "log")]
#[macro_use]
extern crate log;

use bottom::{app::event::EventResult, canvas, options::*, *};

use std::{
    boxed::Box,
    io::stdout,
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

    // Create "app" struct, which will control most of the program and store settings/state
    let mut app = build_app(&matches, &mut config)?;

    // Create painter and set colours.
    let mut painter = canvas::Painter::init(&config, get_color_scheme(&matches, &config)?)?;

    // Create termination mutex and cvar
    #[allow(clippy::mutex_atomic)]
    let thread_termination_lock = Arc::new(Mutex::new(false));
    let thread_termination_cvar = Arc::new(Condvar::new());

    // Set up input handling
    let (sender, receiver) = mpsc::channel();
    let input_thread = create_input_thread(sender.clone(), thread_termination_lock.clone());

    // Cleaning loop
    // TODO: Probably worth spinning this off into an async thread or something...
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
    // TODO: Add back collection sender
    let (_collection_sender, collection_thread_ctrl_receiver) = mpsc::channel();
    let _collection_thread = create_collection_thread(
        sender,
        collection_thread_ctrl_receiver,
        thread_termination_lock.clone(),
        thread_termination_cvar.clone(),
        &app.app_config_fields,
        app.filters.clone(),
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
    // TODO: Make this close all the child threads too!
    panic::set_hook(Box::new(|info| panic_hook(info)));

    // Set termination hook
    let is_terminated = Arc::new(AtomicBool::new(false));
    let ist_clone = is_terminated.clone();
    ctrlc::set_handler(move || {
        ist_clone.store(true, Ordering::SeqCst);
    })?;

    // Paint once first.
    try_drawing(&mut terminal, &mut app, &mut painter)?;

    while !is_terminated.load(Ordering::SeqCst) {
        if let Ok(recv) = receiver.recv() {
            match app.handle_event(recv) {
                EventResult::Quit => {
                    break;
                }
                EventResult::Redraw => {
                    try_drawing(&mut terminal, &mut app, &mut painter)?;
                }
                EventResult::NoRedraw => {
                    continue;
                }
            }
        } else {
            break;
        }
    }

    // I think doing it in this order is safe...
    *thread_termination_lock.lock().unwrap() = true;
    thread_termination_cvar.notify_all();

    let _ = input_thread.join();
    cleanup_terminal(&mut terminal)?;

    Ok(())
}
