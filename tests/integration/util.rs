use std::{env, ffi::OsString, path::Path, process::Command};

use hashbrown::HashMap;
#[cfg(all(target_arch = "x86_64", target_os = "linux"))]
use portable_pty::{Child, CommandBuilder, MasterPty, PtySize, native_pty_system};

pub fn abs_path(path: &str) -> OsString {
    let path = Path::new(path);

    if path.exists() {
        path.canonicalize().unwrap().into_os_string()
    } else {
        // We are going to trust that the path given is valid...
        path.to_owned().into_os_string()
    }
}

/// Returns a QEMU runner target given an architecture.
fn get_qemu_target(arch: &str) -> &str {
    match arch {
        "armv7" => "arm",
        "i686" => "i386",
        "powerpc" => "ppc",
        "powerpc64le" => "ppc64le",
        _ => arch,
    }
}

/// This is required since running binary tests via cross can cause be tricky!
/// We need to basically "magically" grab the correct runner in some cases,
/// which can be done by inspecting env variables that should only show up while
/// using cross.
///
/// Originally inspired by [ripgrep's test files](https://cs.github.com/BurntSushi/ripgrep/blob/9f0e88bcb14e02da1b88872435b17d74786640b5/tests/util.rs#L470),
/// but adapted to work more generally with the architectures supported by
/// bottom after looking through cross' [linux-runner](https://github.com/cross-rs/cross/blob/main/docker/linux-runner) file.
fn cross_runner() -> Option<String> {
    const TARGET_RUNNER: &str = "CARGO_TARGET_RUNNER";
    const CROSS_RUNNER: &str = "CROSS_RUNNER";

    let env_mapping = env::vars_os()
        .filter_map(|(k, v)| {
            let (k, v) = (k.to_string_lossy(), v.to_string_lossy());

            if k.starts_with("CARGO_TARGET_") && k.ends_with("_RUNNER") && !v.is_empty() {
                Some((TARGET_RUNNER.to_string(), v.to_string()))
            } else if k == CROSS_RUNNER && !v.is_empty() {
                Some((k.to_string(), v.to_string()))
            } else {
                None
            }
        })
        .collect::<HashMap<_, _>>();

    if let Some(cross_runner) = env_mapping.get(CROSS_RUNNER) {
        if cross_runner == "qemu-user" {
            env_mapping.get(TARGET_RUNNER).map(|target_runner| {
                format!(
                    "qemu-{}",
                    get_qemu_target(target_runner.split_ascii_whitespace().last().unwrap())
                )
            })
        } else {
            None
        }
    } else {
        env_mapping.get(TARGET_RUNNER).cloned()
    }
}

const BTM_EXE_PATH: &str = env!("CARGO_BIN_EXE_btm");
const RUNNER_ENV_VARS: [(&str, &str); 1] = [("NO_COLOR", "1")];
const DEFAULT_CFG: [&str; 2] = ["-C", "./tests/valid_configs/empty_config.toml"];

/// Returns the [`Command`] of a binary invocation of bottom, alongside
/// any required env variables.
pub fn btm_command(args: &[&str]) -> Command {
    let mut cmd = match cross_runner() {
        None => Command::new(BTM_EXE_PATH),
        Some(runner) => {
            let mut cmd = Command::new(runner);
            cmd.envs(RUNNER_ENV_VARS);
            cmd.arg(BTM_EXE_PATH);
            cmd
        }
    };

    let mut prev = "";
    for arg in args.iter() {
        if prev == "-C" {
            // This is the config file; make sure we set it to absolute path!
            cmd.arg(abs_path(arg));
        } else {
            cmd.arg(arg);
        }

        prev = arg;
    }

    cmd
}

/// Returns the [`Command`] of a binary invocation of bottom, alongside
/// any required env variables, and with the default, empty config file.
pub fn no_cfg_btm_command() -> Command {
    btm_command(&DEFAULT_CFG)
}

/// Spawns `btm` in a pty, returning the pair alongside a handle to the child.
#[cfg(all(target_arch = "x86_64", target_os = "linux"))]
pub fn spawn_btm_in_pty(args: &[&str]) -> (Box<dyn MasterPty>, Box<dyn Child>) {
    let native_pty = native_pty_system();

    let pair = native_pty
        .openpty(PtySize {
            rows: 100,
            cols: 100,
            pixel_width: 1,
            pixel_height: 1,
        })
        .unwrap();

    let btm_exe = BTM_EXE_PATH;
    let mut cmd = match cross_runner() {
        None => CommandBuilder::new(btm_exe),
        Some(runner) => {
            let mut cmd = CommandBuilder::new(runner);
            for (env, val) in RUNNER_ENV_VARS {
                cmd.env(env, val);
            }
            cmd.arg(BTM_EXE_PATH);

            cmd
        }
    };

    let args = if args.is_empty() { &DEFAULT_CFG } else { args };
    let mut prev = "";
    for arg in args.iter() {
        if prev == "-C" {
            // This is the config file; make sure we set it to absolute path!
            cmd.arg(abs_path(arg));
        } else {
            cmd.arg(arg);
        }

        prev = arg;
    }

    (pair.master, pair.slave.spawn_command(cmd).unwrap())
}
