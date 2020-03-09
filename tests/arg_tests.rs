use std::process::Command;

use assert_cmd::prelude::*;
use predicates::prelude::*;

// These tests are mostly here just to ensure that invalid results will be caught when passing arguments...

// TODO: [TEST] Allow for release testing.  Do this with paths.

//======================RATES======================//

fn get_os_binary_loc() -> String {
    if cfg!(target_os = "linux") {
        "./target/x86_64-unknown-linux-gnu/debug/btm".to_string()
    } else if cfg!(target_os = "windows") {
        "./target/x86_64-pc-windows-msvc/debug/btm".to_string()
    } else if cfg!(target_os = "macos") {
        "./target/x86_64-apple-darwin/debug/btm".to_string()
    } else {
        "".to_string()
    }
}

#[test]
fn test_small_rate() -> Result<(), Box<dyn std::error::Error>> {
    Command::new(get_os_binary_loc())
        .arg("-r")
        .arg("249")
        .assert()
        .failure()
        .stderr(predicate::str::contains(
            "Please set your update rate to be at least 250 milliseconds.",
        ));
    Ok(())
}

#[test]
fn test_large_default_time() -> Result<(), Box<dyn std::error::Error>> {
    Command::new(get_os_binary_loc())
        .arg("-t")
        .arg("18446744073709551616")
        .assert()
        .failure()
        .stderr(predicate::str::contains(
            "Please set your default value to be at most unsigned INT_MAX.",
        ));
    Ok(())
}

#[test]
fn test_small_default_time() -> Result<(), Box<dyn std::error::Error>> {
    Command::new(get_os_binary_loc())
        .arg("-t")
        .arg("900")
        .assert()
        .failure()
        .stderr(predicate::str::contains(
            "Please set your default value to be at least 30 seconds.",
        ));
    Ok(())
}

#[test]
fn test_large_delta_time() -> Result<(), Box<dyn std::error::Error>> {
    Command::new(get_os_binary_loc())
        .arg("-d")
        .arg("18446744073709551616")
        .assert()
        .failure()
        .stderr(predicate::str::contains(
            "Please set your time delta to be at most unsigned INT_MAX.",
        ));
    Ok(())
}

#[test]
fn test_small_delta_time() -> Result<(), Box<dyn std::error::Error>> {
    Command::new(get_os_binary_loc())
        .arg("-d")
        .arg("900")
        .assert()
        .failure()
        .stderr(predicate::str::contains(
            "Please set your time delta to be at least 1 second.",
        ));
    Ok(())
}

#[test]
fn test_large_rate() -> Result<(), Box<dyn std::error::Error>> {
    Command::new(get_os_binary_loc())
        .arg("-r")
        .arg("18446744073709551616")
        .assert()
        .failure()
        .stderr(predicate::str::contains(
            "Please set your update rate to be at most unsigned INT_MAX.",
        ));
    Ok(())
}

#[test]
fn test_negative_rate() -> Result<(), Box<dyn std::error::Error>> {
    // This test should auto fail due to how clap works
    Command::new(get_os_binary_loc())
        .arg("-r")
        .arg("-1000")
        .assert()
        .failure()
        .stderr(predicate::str::contains(
            "wasn't expected, or isn't valid in this context",
        ));

    Ok(())
}

#[test]
fn test_invalid_rate() -> Result<(), Box<dyn std::error::Error>> {
    Command::new(get_os_binary_loc())
        .arg("-r")
        .arg("100-1000")
        .assert()
        .failure()
        .stderr(predicate::str::contains("invalid digit"));

    Ok(())
}

#[test]
fn test_conflicting_temps() -> Result<(), Box<dyn std::error::Error>> {
    Command::new(get_os_binary_loc())
        .arg("-c")
        .arg("-f")
        .assert()
        .failure()
        .stderr(predicate::str::contains(
            "cannot be used with one or more of the other specified arguments",
        ));

    Ok(())
}

#[test]
fn test_conflicting_default_widget() -> Result<(), Box<dyn std::error::Error>> {
    Command::new(get_os_binary_loc())
        .arg("--cpu_default")
        .arg("--disk_default")
        .assert()
        .failure()
        .stderr(predicate::str::contains(
            "cannot be used with one or more of the other specified arguments",
        ));

    Ok(())
}
