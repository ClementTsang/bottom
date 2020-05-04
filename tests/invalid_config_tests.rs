use assert_cmd::prelude::*;
use predicates::prelude::*;
use std::process::Command;

// These tests are for testing some config file-specific options.

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
fn test_toml_mismatch_type() -> Result<(), Box<dyn std::error::Error>> {
    Command::new(get_binary_location())
        .arg("-C")
        .arg("./tests/invalid_configs/toml_mismatch_type.toml")
        .assert()
        .failure()
        .stderr(predicate::str::contains("invalid type"));
    Ok(())
}

#[test]
fn test_empty_layout() -> Result<(), Box<dyn std::error::Error>> {
    Command::new(get_binary_location())
        .arg("-C")
        .arg("./tests/invalid_configs/empty_layout.toml")
        .assert()
        .failure()
        .stderr(predicate::str::contains("invalid layout config"));
    Ok(())
}

#[test]
fn test_invalid_layout_widget_type() -> Result<(), Box<dyn std::error::Error>> {
    Command::new(get_binary_location())
        .arg("-C")
        .arg("./tests/invalid_configs/invalid_layout_widget_type.toml")
        .assert()
        .failure()
        .stderr(predicate::str::contains("invalid widget type"));
    Ok(())
}

/// This test isn't really needed as this is technically covered by TOML spec.
/// However, I feel like it's worth checking anyways - not like it takes long.
#[test]
fn test_duplicate_temp_type() -> Result<(), Box<dyn std::error::Error>> {
    Command::new(get_binary_location())
        .arg("-C")
        .arg("./tests/invalid_configs/duplicate_temp_type.toml")
        .assert()
        .failure()
        .stderr(predicate::str::contains("duplicate field"));
    Ok(())
}

/// Checks for if a hex is valid
#[test]
fn test_invalid_colour_hex() -> Result<(), Box<dyn std::error::Error>> {
    Command::new(get_binary_location())
        .arg("-C")
        .arg("./tests/invalid_configs/invalid_colour_hex.toml")
        .assert()
        .failure()
        .stderr(predicate::str::contains("invalid color hex"));
    Ok(())
}

/// Checks for if a hex is too long
#[test]
fn test_invalid_colour_hex_2() -> Result<(), Box<dyn std::error::Error>> {
    Command::new(get_binary_location())
        .arg("-C")
        .arg("./tests/invalid_configs/invalid_colour_hex_2.toml")
        .assert()
        .failure()
        .stderr(predicate::str::contains("invalid color hex"));
    Ok(())
}

/// Checks unicode hex because the way we originally did it could cause char
/// boundary errors!
#[test]
fn test_invalid_colour_hex_3() -> Result<(), Box<dyn std::error::Error>> {
    Command::new(get_binary_location())
        .arg("-C")
        .arg("./tests/invalid_configs/invalid_colour_hex_3.toml")
        .assert()
        .failure()
        .stderr(predicate::str::contains("invalid color hex"));
    Ok(())
}

#[test]
fn test_invalid_colour_name() -> Result<(), Box<dyn std::error::Error>> {
    Command::new(get_binary_location())
        .arg("-C")
        .arg("./tests/invalid_configs/invalid_colour_name.toml")
        .assert()
        .failure()
        .stderr(predicate::str::contains("invalid named color"));
    Ok(())
}

#[test]
fn test_invalid_colour_rgb() -> Result<(), Box<dyn std::error::Error>> {
    Command::new(get_binary_location())
        .arg("-C")
        .arg("./tests/invalid_configs/invalid_colour_rgb.toml")
        .assert()
        .failure()
        .stderr(predicate::str::contains("invalid RGB color"));
    Ok(())
}

#[test]
fn test_invalid_colour_rgb_2() -> Result<(), Box<dyn std::error::Error>> {
    Command::new(get_binary_location())
        .arg("-C")
        .arg("./tests/invalid_configs/invalid_colour_rgb_2.toml")
        .assert()
        .failure()
        .stderr(predicate::str::contains("invalid RGB color"));
    Ok(())
}

#[test]
fn test_invalid_colour_string() -> Result<(), Box<dyn std::error::Error>> {
    Command::new(get_binary_location())
        .arg("-C")
        .arg("./tests/invalid_configs/invalid_colour_string.toml")
        .assert()
        .failure()
        .stderr(predicate::str::contains("invalid named color"));
    Ok(())
}
