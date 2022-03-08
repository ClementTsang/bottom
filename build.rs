use clap_complete::{generate_to, shells::Shell};
use std::{env, fs, process};

include!("src/clap.rs");

fn main() {
    // OUT_DIR is where extra build files are written to for Cargo.
    let out_dir = match env::var_os("OUT_DIR") {
        Some(out_dir) => out_dir,
        None => {
            eprintln!("The OUT_DIR environment variable was not set!  Aborting...");
            process::exit(1)
        }
    };
    match fs::create_dir_all(&out_dir) {
        Ok(()) => {}
        Err(err) => {
            eprintln!(
                "Failed to create a directory at OUT_DIR location {:?}, encountered error {:?}.  Aborting...",
                out_dir, err
            );
            process::exit(1)
        }
    }

    // Generate completions
    let mut app = build_app();
    generate_to(Shell::Bash, &mut app, "btm", &out_dir).unwrap();
    generate_to(Shell::Zsh, &mut app, "btm", &out_dir).unwrap();
    generate_to(Shell::Fish, &mut app, "btm", &out_dir).unwrap();
    generate_to(Shell::PowerShell, &mut app, "btm", &out_dir).unwrap();
    generate_to(Shell::Elvish, &mut app, "btm", &out_dir).unwrap();
}
