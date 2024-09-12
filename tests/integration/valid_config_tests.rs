//! Tests config files that have sometimes caused issues despite being valid.

use std::{
    io::{Read, Write},
    thread,
    time::Duration,
};

use regex::Regex;

use crate::util::spawn_btm_in_pty;

fn reader_to_string(mut reader: Box<dyn Read>) -> String {
    let mut buf = String::default();
    reader.read_to_string(&mut buf).unwrap();

    buf
}

fn run_and_kill(args: &[&str]) {
    let (master, mut handle) = spawn_btm_in_pty(args);
    let reader = master.try_clone_reader().unwrap();
    let _ = master.take_writer().unwrap();

    const TIMES_CHECKED: u64 = 6; // Check 6 times, once every 500ms, for 3 seconds total.

    for _ in 0..TIMES_CHECKED {
        thread::sleep(Duration::from_millis(500));
        match handle.try_wait() {
            Ok(Some(exit)) => {
                println!("output: {}", reader_to_string(reader));
                panic!("program terminated unexpectedly (exit status: {exit:?})");
            }
            Err(e) => {
                println!("output: {}", reader_to_string(reader));
                panic!("error while trying to wait: {e}")
            }
            _ => {}
        }
    }

    handle.kill().unwrap();
}

#[test]
fn test_basic() {
    run_and_kill(&[]);
}

/// A test to ensure that a bad config will fail the `run_and_kill` function.
#[test]
#[should_panic]
fn test_bad_basic() {
    run_and_kill(&["--this_does_not_exist"]);
}

#[test]
fn test_empty() {
    run_and_kill(&["-C", "./tests/valid_configs/empty_config.toml"]);
}

#[test]
fn test_default() {
    // Take the default config file and uncomment everything.
    let default_config = match std::fs::File::open("./sample_configs/default_config.toml") {
        Ok(mut default_config_file) => {
            let mut buf = String::new();
            default_config_file
                .read_to_string(&mut buf)
                .expect("can read file");

            buf
        }
        Err(err) => {
            println!("Could not open default config, skipping test_default: {err:?}");
            return;
        }
    };

    let default_config = Regex::new(r"(?m)^#([a-zA-Z\[])")
        .unwrap()
        .replace_all(&default_config, "$1");

    let default_config = Regex::new(r"(?m)^#(\s\s+)([a-zA-Z\[])")
        .unwrap()
        .replace_all(&default_config, "$2");

    let Ok(mut uncommented_config) = tempfile::NamedTempFile::new() else {
        println!("Could not create a temp file, skipping test_default.");
        return;
    };

    if uncommented_config
        .write_all(default_config.as_bytes())
        .is_err()
    {
        println!("Could not write to temp file, skipping test_default.");
        return;
    }

    run_and_kill(&["-C", &uncommented_config.path().to_string_lossy()]);
}

#[test]
fn test_many_proc() {
    run_and_kill(&["-C", "./tests/valid_configs/many_proc.toml"]);
}

#[test]
fn test_all_proc() {
    run_and_kill(&["-C", "./tests/valid_configs/all_proc.toml"]);
}

#[test]
fn test_cpu_doughnut() {
    run_and_kill(&["-C", "./tests/valid_configs/cpu_doughnut.toml"]);
}

#[test]
fn test_theme() {
    run_and_kill(&["-C", "./tests/valid_configs/theme.toml"]);
}

#[test]
fn test_styling_sanity_check() {
    run_and_kill(&["-C", "./tests/valid_configs/styling.toml"]);
}

#[test]
fn test_styling_sanity_check_2() {
    run_and_kill(&["-C", "./tests/valid_configs/styling_2.toml"]);
}

#[test]
fn test_filtering() {
    run_and_kill(&["-C", "./tests/valid_configs/filtering.toml"]);
}
