use std::{
    env, fs, io,
    path::{Path, PathBuf},
};

use clap_complete::{generate_to, shells::Shell};

include!("src/clap.rs");

fn create_dir(dir: &Path) -> io::Result<()> {
    let res = fs::create_dir_all(dir);
    match &res {
        Ok(()) => {}
        Err(err) => {
            eprintln!(
            "Failed to create a directory at location {dir:?}, encountered error {err:?}.  Aborting...",
        );
        }
    }

    res
}

fn btm_generate() -> io::Result<()> {
    const ENV_KEY: &str = "BTM_GENERATE";

    match env::var_os(ENV_KEY) {
        Some(var) if !var.is_empty() => {
            const COMPLETION_DIR: &str = "./target/tmp/bottom/completion/";
            const MANPAGE_DIR: &str = "./target/tmp/bottom/manpage/";

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

    println!("cargo:rerun-if-env-changed={ENV_KEY}");

    Ok(())
}

fn nightly_version() {
    const ENV_KEY: &str = "BTM_BUILD_RELEASE_CALLER";

    match env::var_os(ENV_KEY) {
        Some(var) if !var.is_empty() && var == "nightly" => {
            let version = env!("CARGO_PKG_VERSION");

            if let Some(git_hash) = option_env!("CIRRUS_CHANGE_IN_REPO")
                .and_then(|cirrus_sha: &str| cirrus_sha.get(0..8))
            {
                println!("cargo:rustc-env=NIGHTLY_VERSION={version}-nightly-{git_hash}");
            } else if let Some(git_hash) =
                option_env!("GITHUB_SHA").and_then(|gha_sha: &str| gha_sha.get(0..8))
            {
                println!("cargo:rustc-env=NIGHTLY_VERSION={version}-nightly-{git_hash}");
            } else if let Ok(output) = std::process::Command::new("git")
                .args(["rev-parse", "--short=8", "HEAD"])
                .output()
            {
                let git_hash = String::from_utf8(output.stdout).unwrap();
                println!("cargo:rustc-env=NIGHTLY_VERSION={version}-nightly-{git_hash}");
            }
        }
        _ => {}
    }

    println!("cargo:rerun-if-env-changed={ENV_KEY}");
    println!("cargo:rerun-if-env-changed=CIRRUS_CHANGE_IN_REPO");
}

fn main() -> Result<()> {
    btm_generate()?;
    nightly_version();

    Ok(())
}
