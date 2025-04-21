//! General build script used by bottom to generate completion files and set binary version.

#[expect(dead_code)]
#[path = "src/options/args.rs"]
mod args;

use std::{
    env, fs, io,
    path::{Path, PathBuf},
};

use clap::{Command, CommandFactory};
use clap_complete::{Generator, generate_to, shells::Shell};
use clap_complete_fig::Fig;
use clap_complete_nushell::Nushell;

use crate::args::BottomArgs;

fn create_dir(dir: &Path) -> io::Result<()> {
    fs::create_dir_all(dir).inspect_err(|err| {
        eprintln!(
            "Couldn't create a directory at {} ({:?}). Aborting.",
            dir.display(),
            err
        )
    })
}

fn generate_completions<G>(to_generate: G, cmd: &mut Command, out_dir: &Path) -> io::Result<PathBuf>
where
    G: Generator,
{
    generate_to(to_generate, cmd, "btm", out_dir)
}

fn btm_generate() -> io::Result<()> {
    const ENV_KEY: &str = "BTM_GENERATE";

    match env::var_os(ENV_KEY) {
        Some(var) if !var.is_empty() => {
            let completion_dir =
                option_env!("COMPLETION_DIR").unwrap_or("./target/tmp/bottom/completion/");
            let manpage_dir = option_env!("MANPAGE_DIR").unwrap_or("./target/tmp/bottom/manpage/");

            let completion_out_dir = PathBuf::from(completion_dir);
            let manpage_out_dir = PathBuf::from(manpage_dir);

            create_dir(&completion_out_dir)?;
            create_dir(&manpage_out_dir)?;

            // Generate completions
            let mut app = BottomArgs::command();
            generate_completions(Shell::Bash, &mut app, &completion_out_dir)?;
            generate_completions(Shell::Zsh, &mut app, &completion_out_dir)?;
            generate_completions(Shell::Fish, &mut app, &completion_out_dir)?;
            generate_completions(Shell::PowerShell, &mut app, &completion_out_dir)?;
            generate_completions(Shell::Elvish, &mut app, &completion_out_dir)?;
            generate_completions(Fig, &mut app, &completion_out_dir)?;
            generate_completions(Nushell, &mut app, &completion_out_dir)?;

            // Generate manpage
            let app = app.name("btm");
            let man = clap_mangen::Man::new(app);
            let mut buffer: Vec<u8> = Default::default();
            man.render(&mut buffer)?;
            fs::write(manpage_out_dir.join("btm.1"), buffer)?;
        }
        _ => {}
    }

    println!("cargo:rerun-if-env-changed={ENV_KEY}");

    Ok(())
}

fn extract_sha(sha: Option<&str>) -> Option<&str> {
    sha.and_then(|sha: &str| sha.get(0..8))
}

fn output_nightly_version(version: &str, git_hash: &str) {
    println!("cargo:rustc-env=NIGHTLY_VERSION={version}-nightly-{git_hash}");
}

fn nightly_version() {
    const ENV_KEY: &str = "BTM_BUILD_RELEASE_CALLER";

    match env::var_os(ENV_KEY) {
        Some(var) if !var.is_empty() && var == "ci" => {
            let version = env!("CARGO_PKG_VERSION");

            if let Some(hash) = extract_sha(option_env!("CIRRUS_CHANGE_IN_REPO")) {
                // May be set if we're building with Cirrus CI.
                output_nightly_version(version, hash);
            } else if let Some(hash) = extract_sha(option_env!("GITHUB_SHA")) {
                // May be set if we're building with GHA.
                output_nightly_version(version, hash);
            } else if let Ok(output) = std::process::Command::new("git")
                .args(["rev-parse", "--short=8", "HEAD"])
                .output()
            {
                // If we're not building in either, we do the lazy thing and fall back to
                // manually grabbing info using git as a command.
                let hash = String::from_utf8(output.stdout).unwrap();
                output_nightly_version(version, &hash);
            }
        }
        _ => {}
    }

    println!("cargo:rerun-if-env-changed={ENV_KEY}");
    println!("cargo:rerun-if-env-changed=CIRRUS_CHANGE_IN_REPO");
}

fn main() -> io::Result<()> {
    btm_generate()?;
    nightly_version();

    Ok(())
}
