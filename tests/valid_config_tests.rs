//! Tests config files that have sometimes caused issues despite being valid.

mod util;

use std::{process::Stdio, thread, time::Duration};

use util::*;

fn run_and_kill(args: &[&str]) {
    let mut handle = btm_command()
        .stdout(Stdio::piped())
        .args(args)
        .spawn()
        .unwrap();

    thread::sleep(Duration::from_millis(2000));

    match handle.try_wait() {
        Ok(Some(exit)) => panic!("program terminated before it should have - exit code {exit:?}"),
        Err(e) => panic!("error while trying to wait: {e}"),
        _ => {}
    }

    handle.kill().unwrap();
}

#[test]
fn test_basic() {
    run_and_kill(&[]);
}

#[test]
fn test_many_proc() {
    run_and_kill(&["-C", "./tests/valid_configs/many_proc.toml"]);
}
