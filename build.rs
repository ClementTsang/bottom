use clap_complete::{generate_to, shells::Shell};
use std::{
    env, fs,
    io::Result,
    path::{Path, PathBuf},
};

include!("src/clap.rs");

fn create_dir(dir: &Path) -> Result<()> {
    let res = fs::create_dir_all(&dir);
    match &res {
        Ok(()) => {}
        Err(err) => {
            eprintln!(
            "Failed to create a directory at location {:?}, encountered error {:?}.  Aborting...",
            dir, err
        );
        }
    }

    res
}

fn main() -> Result<()> {
    if env::var_os("GENERATE").is_some() {
        // OUT_DIR is where extra build files are written to for Cargo.
        let completion_out_dir = PathBuf::from("completion");
        let manpage_out_dir = PathBuf::from("manpage");

        create_dir(&completion_out_dir)?;
        create_dir(&manpage_out_dir)?;

        // Generate completions
        let mut app = build_app();
        generate_to(Shell::Bash, &mut app, "btm", &completion_out_dir)?;
        generate_to(Shell::Zsh, &mut app, "btm", &completion_out_dir)?;
        generate_to(Shell::Fish, &mut app, "btm", &completion_out_dir)?;
        generate_to(Shell::PowerShell, &mut app, "btm", &completion_out_dir)?;
        generate_to(Shell::Elvish, &mut app, "btm", &completion_out_dir)?;

        // Generate manpage
        let app = app.name("btm");
        let man = clap_mangen::Man::new(app);
        let mut buffer: Vec<u8> = Default::default();
        man.render(&mut buffer)?;
        std::fs::write(manpage_out_dir.join("btm.1"), buffer)?;
    }

    println!("cargo:rerun-if-changed=build.rs");
    println!("cargo:rerun-if-changed=./src/clap.rs");
    println!("cargo:rerun-if-env-changed=GENERATE");

    Ok(())
}
