extern crate clap;

use clap::Shell;
use std::env;

include!("src/clap.rs");

fn main() {
    let out_dir = match env::var_os("OUT_DIR") {
        None => return,
        Some(out_dir) => out_dir,
    };
    let mut app = build_matches();
    app.gen_completions("btm", Shell::Bash, &out_dir);
    app.gen_completions("btm", Shell::Zsh, &out_dir);
    app.gen_completions("btm", Shell::Fish, &out_dir);
    app.gen_completions("btm", Shell::PowerShell, &out_dir);
}
