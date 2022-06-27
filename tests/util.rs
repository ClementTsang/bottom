use std::{env, process::Command};

/// From [ripgrep's test files](https://cs.github.com/BurntSushi/ripgrep/blob/9f0e88bcb14e02da1b88872435b17d74786640b5/tests/util.rs#L470).
///
/// This is required since running binary tests via cross can cause some problems.
fn cross_runner() -> Option<String> {
    for (k, v) in std::env::vars_os() {
        let (k, v) = (k.to_string_lossy(), v.to_string_lossy());
        if !(k.starts_with("CARGO_TARGET_") && k.ends_with("_RUNNER")) {
            continue;
        }
        // if !v.starts_with("qemu-") {
        //     continue;
        // }
        return Some(v.into_owned());
    }
    None
}

/// Returns the [`Command`] of a binary invocation of bottom.
///
pub fn btm_command() -> Command {
    let btm_exe = env!("CARGO_BIN_EXE_btm");
    match cross_runner() {
        None => Command::new(btm_exe),
        Some(runner) => {
            let mut cmd = Command::new(runner);
            cmd.arg(btm_exe);
            cmd
        }
    }
}
