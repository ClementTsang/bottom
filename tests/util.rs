use std::{env, process::Command};

use hashbrown::HashMap;

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

/// This is required since running binary tests via cross can cause be tricky! We need to basically "magically" grab
/// the correct runner in some cases, which can be done by inspecting env variables that should only show up while
/// using cross.
///
/// Originally inspired by [ripgrep's test files](https://cs.github.com/BurntSushi/ripgrep/blob/9f0e88bcb14e02da1b88872435b17d74786640b5/tests/util.rs#L470),
/// but adapted to work more generally with the architectures supported by bottom after looking through cross'
/// [linux-runner](https://github.com/cross-rs/cross/blob/main/docker/linux-runner) file.
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
                    get_qemu_target(
                        target_runner
                            .split_ascii_whitespace()
                            .collect::<Vec<_>>()
                            .last()
                            .unwrap()
                    )
                )
            })
        } else {
            None
        }
    } else {
        env_mapping.get(TARGET_RUNNER).cloned()
    }
}

/// Returns the [`Command`] of a binary invocation of bottom, alongside
/// any required env variables.
pub fn btm_command() -> Command {
    let btm_exe = env!("CARGO_BIN_EXE_btm");
    match cross_runner() {
        None => Command::new(btm_exe),
        Some(runner) => {
            let mut cmd = Command::new(runner);
            cmd.env("NO_COLOR", "1");
            cmd.arg(btm_exe);
            cmd
        }
    }
}
