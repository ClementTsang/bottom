//! Tests config files that have sometimes caused issues despite being valid.

use std::{io::Read, thread, time::Duration};
#[cfg(feature = "default")]
use std::{io::Write, path::Path};

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

#[cfg(feature = "default")]
fn test_uncommented_default_config(original: &Path, test_name: &str) {
    use regex::Regex;

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

#[cfg(feature = "default")]
#[test]
fn test_default() {
    test_uncommented_default_config(
        Path::new("./sample_configs/default_config.toml"),
        "test_default",
    );
}

#[cfg(feature = "default")]
#[test]
fn test_new_default() {
    use tempfile::TempPath;

    let new_temp_default_path = match tempfile::NamedTempFile::new() {
        Ok(temp_file) => temp_file.into_temp_path(),
        Err(err) => {
            println!("Could not create a temp file, skipping test_new_default. Error: {err:?}");
            return;
        }
    };

    // This is a hack because we need a temp file that doesn't exist.
    let actual_temp_default_path = new_temp_default_path.to_path_buf();
    new_temp_default_path.close().unwrap();

    if !actual_temp_default_path.exists() {
        run_and_kill(&["-C", &(actual_temp_default_path.to_string_lossy())]);

        // Re-take control over the temp path to ensure it gets deleted.
        let actual_temp_default_path = TempPath::from_path(actual_temp_default_path);
        test_uncommented_default_config(&actual_temp_default_path, "test_new_default");

        actual_temp_default_path.close().unwrap();
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

#[test]
fn test_proc_columns() {
    run_and_kill(&["-C", "./tests/valid_configs/proc_columns.toml"]);
}

#[cfg(target_os = "linux")]
#[test]
fn test_linux_only() {
    run_and_kill(&["-C", "./tests/valid_configs/os_specific/linux.toml"]);
}
