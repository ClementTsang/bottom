use assert_cmd::prelude::*;
use predicates::prelude::*;
use std::process::Command;

// These tests are mostly here just to ensure that invalid results will be caught when passing arguments...

//======================RATES======================//

fn get_binary_location() -> String {
    // env!("CARGO_BIN_EXE_btm").to_string()
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
    Command::new(get_binary_location())
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
    Command::new(get_binary_location())
        .arg("-t")
        .arg("18446744073709551616")
        .assert()
        .failure()
        .stderr(predicate::str::contains(
            "Please set your default value to be at most",
        ));
    Ok(())
}

#[test]
fn test_small_default_time() -> Result<(), Box<dyn std::error::Error>> {
    Command::new(get_binary_location())
        .arg("-t")
        .arg("900")
        .assert()
        .failure()
        .stderr(predicate::str::contains(
            "Please set your default value to be at least",
        ));
    Ok(())
}

#[test]
fn test_large_delta_time() -> Result<(), Box<dyn std::error::Error>> {
    Command::new(get_binary_location())
        .arg("-d")
        .arg("18446744073709551616")
        .assert()
        .failure()
        .stderr(predicate::str::contains(
            "Please set your time delta to be at most",
        ));
    Ok(())
}

#[test]
fn test_small_delta_time() -> Result<(), Box<dyn std::error::Error>> {
    Command::new(get_binary_location())
        .arg("-d")
        .arg("900")
        .assert()
        .failure()
        .stderr(predicate::str::contains(
            "Please set your time delta to be at least",
        ));
    Ok(())
}

#[test]
fn test_large_rate() -> Result<(), Box<dyn std::error::Error>> {
    Command::new(get_binary_location())
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
    Command::new(get_binary_location())
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
    Command::new(get_binary_location())
        .arg("-r")
        .arg("100-1000")
        .assert()
        .failure()
        .stderr(predicate::str::contains("invalid digit"));

    Ok(())
}

#[test]
fn test_conflicting_temps() -> Result<(), Box<dyn std::error::Error>> {
    Command::new(get_binary_location())
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
fn test_invalid_default_widget_1() -> Result<(), Box<dyn std::error::Error>> {
    Command::new(get_binary_location())
        .arg("--default_widget_type")
        .arg("fake_widget")
        .assert()
        .failure()
        .stderr(predicate::str::contains("invalid widget type"));

    Ok(())
}

#[test]
fn test_invalid_default_widget_2() -> Result<(), Box<dyn std::error::Error>> {
    Command::new(get_binary_location())
        .arg("--default_widget_type")
        .arg("cpu")
        .arg("--default_widget_count")
        .arg("18446744073709551616")
        .assert()
        .failure()
        .stderr(predicate::str::contains(
            "Please set your widget count to be at most unsigned INT_MAX",
        ));

    Ok(())
}
