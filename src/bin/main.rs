#![warn(rust_2018_idioms)]
#[allow(unused_imports)]
#[cfg(feature = "log")]
#[macro_use]
extern crate log;

use bottom::{app::AppMessages, options::*, tuine::RuntimeEvent, *};

use std::{
    boxed::Box,
    panic,
    sync::{mpsc, Arc, Condvar, Mutex},
    thread,
    time::Duration,
};

use anyhow::{Context, Result};

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
    let app = build_app(&matches, &mut config)?;

    // Create termination mutex and cvar
    #[allow(clippy::mutex_atomic)]
    let thread_termination_lock = Arc::new(Mutex::new(false));
    let thread_termination_cvar = Arc::new(Condvar::new());

    // Set up input handling
    let (sender, receiver) = mpsc::channel();
    let input_thread = create_input_thread(sender.clone(), thread_termination_lock.clone());

    // Cleaning loop
    // TODO: [Refactor, Optimization (Potentially, maybe not)] Probably worth spinning this off into an async thread or something...
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
                if cleaning_sender
                    .send(RuntimeEvent::Custom(AppMessages::Clean))
                    .is_err()
                {
                    // debug!("Failed to send cleaning sender...");
                    break;
                }
            }
        })
    };

    // Event loop
    // TODO: [Threads, Refactor, Config] Add back collection sender for config later if we need to change settings on the fly
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
    let mut terminal = init_terminal()?;

    // Set panic hook
    // TODO: [Threads, Panic] Make this close all the child threads too!
    panic::set_hook(Box::new(|info| panic_hook(info)));

    tuine::launch_with_application(app, receiver, &mut terminal)?; // FIXME: Move terminal construction INSIDE

    // I think doing it in this order is safe...
    *thread_termination_lock.lock().unwrap() = true;
    thread_termination_cvar.notify_all();

    let _ = input_thread.join();
    cleanup_terminal(&mut terminal);

    Ok(())
}
