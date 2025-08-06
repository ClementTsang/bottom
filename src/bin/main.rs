//! Main entrypoint for the application.

use bottom::{reset_stdout, start_bottom};

fn main() -> anyhow::Result<()> {
    let mut run_error_hook = false;

    start_bottom(&mut run_error_hook).inspect_err(|_| {
        if run_error_hook {
            reset_stdout();
        }
    })
}
