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
    const COMPLETION_DIR: &str = "./target/tmp/bottom/completion/";
    const MANPAGE_DIR: &str = "./target/tmp/bottom/manpage/";

    match env::var_os("BTM_GENERATE") {
        Some(var) if !var.is_empty() => {
            let completion_out_dir = PathBuf::from(COMPLETION_DIR);
            let manpage_out_dir = PathBuf::from(MANPAGE_DIR);

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
        _ => {}
    }

    println!("cargo:rerun-if-env-changed=BTM_GENERATE");

    Ok(())
}
