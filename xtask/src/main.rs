use clap_complete::{generate_to, shells::Shell};
use std::{env, fs, io::Result, path::PathBuf, process};

include!("../../src/clap.rs");

fn main() -> Result<()> {
    let command = if let Some(command) = env::args().nth(1) {
        command
    } else {
        eprintln!("A command was not given!");
        process::exit(1)
    };

    let out_dir = if let Some(out_dir) = env::args().nth(2) {
        PathBuf::from(out_dir)
    } else {
        eprintln!("An output directory was not set!");
        process::exit(1)
    };

    if let Err(err) = fs::create_dir_all(&out_dir) {
        eprintln!(
            "Failed to create a directory at the output director location {:?}, encountered error {:?}.  Aborting...",
            out_dir, err
        );
        process::exit(1)
    }

    match command.as_str() {
        "completion" => {
            // Generate completions
            let mut app = build_app();
            generate_to(Shell::Bash, &mut app, "btm", &out_dir)?;
            generate_to(Shell::Zsh, &mut app, "btm", &out_dir)?;
            generate_to(Shell::Fish, &mut app, "btm", &out_dir)?;
            generate_to(Shell::PowerShell, &mut app, "btm", &out_dir)?;
            generate_to(Shell::Elvish, &mut app, "btm", &out_dir)?;
        }
        "manpage" => {
            // Generate manpage
            let app = build_app();
            let man = clap_mangen::Man::new(app);
            let mut buffer: Vec<u8> = Default::default();
            man.render(&mut buffer).unwrap();

            std::fs::write(out_dir.join("btm.1"), buffer)?;
        }
        _ => {
            eprintln!("Invalid command given: `{}`", command);
            process::exit(1)
        }
    }

    Ok(())
}
