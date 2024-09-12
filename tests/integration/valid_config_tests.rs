//! Tests config files that have sometimes caused issues despite being valid.

use std::{
    io::{Read, Write},
    path::Path,
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

fn test_uncommented_default_config(original: &Path, test_name: &str) {
    // Take the default config file and uncomment everything.
    let default_config = match std::fs::File::open(original) {
        Ok(mut default_config_file) => {
            let mut buf = String::new();
            default_config_file
                .read_to_string(&mut buf)
                .expect("can read file");

            buf
        }
        Err(err) => {
            println!("Could not open default config, skipping {test_name}. Error: {err:?}");
            return;
        }
    };

    println!("default config: {default_config}");

    let default_config = Regex::new(r"(?m)^#([a-zA-Z\[])")
        .unwrap()
        .replace_all(&default_config, "$1");

    let default_config = Regex::new(r"(?m)^#(\s\s+)([a-zA-Z\[])")
        .unwrap()
        .replace_all(&default_config, "$2");

    let mut uncommented_config = match tempfile::NamedTempFile::new() {
        Ok(tf) => tf,
        Err(err) => {
            println!("Could not create a temp file, skipping {test_name}. Error: {err:?}");
            return;
        }
    };

    if let Err(err) = uncommented_config.write_all(default_config.as_bytes()) {
        println!("Could not write to temp file, skipping {test_name}. Error: {err:?}");
        return;
    }

    run_and_kill(&["-C", &uncommented_config.path().to_string_lossy()]);

    uncommented_config.close().unwrap();
}

#[test]
fn test_default() {
    test_uncommented_default_config(
        Path::new("./sample_configs/default_config.toml"),
        "test_default",
    );
}

#[test]
fn test_new_default() {
    let new_temp_default_path = match tempfile::NamedTempFile::new() {
        Ok(temp_file) => temp_file.into_temp_path(),
        Err(err) => {
            println!("Could not create a temp file, skipping test_new_default. Error: {err:?}");
            return;
        }
    };

    // This is a hack because we need a file that doesn't exist, and this hopefully means we avoid a bit of TOCTOU...?
    let actual_temp_default_path = new_temp_default_path.join("_test_test_test_test");
    new_temp_default_path.close().unwrap();

    if !actual_temp_default_path.exists() {
        run_and_kill(&["-C", &(actual_temp_default_path.to_string_lossy())]);
        test_uncommented_default_config(&actual_temp_default_path, "test_new_default");
    } else {
        println!("temp path we want to check exists, skip test_new_default test.");
    }
}

#[test]
fn test_demo() {
    let path: &str = "./sample_configs/demo_config.toml";
    if std::path::Path::new(path).exists() {
        run_and_kill(&["-C", path]);
    } else {
        println!("Could not read demo config.");
    }
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
