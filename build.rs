use clap_complete::{generate_to, shells::Shell};
use std::{env, fs, io::Result, path::PathBuf, process};

include!("src/clap.rs");

fn main() -> Result<()> {
    if Ok("release".to_owned()) == env::var("PROFILE") {
        // OUT_DIR is where extra build files are written to for Cargo.
        let out_dir = if let Some(out_dir) = env::var_os("OUT_DIR") {
            PathBuf::from(out_dir)
        } else {
            eprintln!("The OUT_DIR environment variable was not set!  Aborting...");
            process::exit(1)
        };

        if let Err(err) = fs::create_dir_all(&out_dir) {
            eprintln!(
            "Failed to create a directory at OUT_DIR location {:?}, encountered error {:?}.  Aborting...",
            out_dir, err
        );
            process::exit(1)
        }

        // Generate completions
        let mut app = build_app();
        generate_to(Shell::Bash, &mut app, "btm", &out_dir)?;
        generate_to(Shell::Zsh, &mut app, "btm", &out_dir)?;
        generate_to(Shell::Fish, &mut app, "btm", &out_dir)?;
        generate_to(Shell::PowerShell, &mut app, "btm", &out_dir)?;
        generate_to(Shell::Elvish, &mut app, "btm", &out_dir)?;

        // Generate manpage
        // let man = clap_mangen::Man::new(app);
        // let mut buffer: Vec<u8> = Default::default();
        // man.render(&mut buffer).unwrap();

        // std::fs::write(out_dir.join("btm.1"), buffer)?;
    }

    Ok(())
}
