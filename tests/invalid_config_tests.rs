use assert_cmd::prelude::*;
use predicates::prelude::*;
use std::process::Command;

// These tests are for testing some config file-specific options.

fn get_binary_location() -> String {
    env!("CARGO_BIN_EXE_btm").to_string()
}

#[test]
fn test_toml_mismatch_type() {
    Command::new(get_binary_location())
        .arg("-C")
        .arg("./tests/invalid_configs/toml_mismatch_type.toml")
        .assert()
        .failure()
        .stderr(predicate::str::contains("invalid type"));
}

#[test]
fn test_empty_layout() {
    Command::new(get_binary_location())
        .arg("-C")
        .arg("./tests/invalid_configs/empty_layout.toml")
        .assert()
        .failure()
        .stderr(predicate::str::contains("at least one widget"));
}

#[test]
fn test_invalid_layout_widget_type() {
    Command::new(get_binary_location())
        .arg("-C")
        .arg("./tests/invalid_configs/invalid_layout_widget_type.toml")
        .assert()
        .failure()
        .stderr(predicate::str::contains("invalid widget name"));
}

/// This test isn't really needed as this is technically covered by TOML spec.
/// However, I feel like it's worth checking anyways - not like it takes long.
#[test]
fn test_duplicate_temp_type() {
    Command::new(get_binary_location())
        .arg("-C")
        .arg("./tests/invalid_configs/duplicate_temp_type.toml")
        .assert()
        .failure()
        .stderr(predicate::str::contains("duplicate field"));
}

/// Checks for if a hex is valid
#[test]
fn test_invalid_colour_hex() {
    Command::new(get_binary_location())
        .arg("-C")
        .arg("./tests/invalid_configs/invalid_colour_hex.toml")
        .assert()
        .failure()
        .stderr(predicate::str::contains("invalid hex colour"));
}

/// Checks for if a hex is too long
#[test]
fn test_invalid_colour_hex_2() {
    Command::new(get_binary_location())
        .arg("-C")
        .arg("./tests/invalid_configs/invalid_colour_hex_2.toml")
        .assert()
        .failure()
        .stderr(predicate::str::contains("invalid hex colour"));
}

/// Checks unicode hex because the way we originally did it could cause char
/// boundary errors!
#[test]
fn test_invalid_colour_hex_3() {
    Command::new(get_binary_location())
        .arg("-C")
        .arg("./tests/invalid_configs/invalid_colour_hex_3.toml")
        .assert()
        .failure()
        .stderr(predicate::str::contains("invalid hex colour"));
}

#[test]
fn test_invalid_colour_name() {
    Command::new(get_binary_location())
        .arg("-C")
        .arg("./tests/invalid_configs/invalid_colour_name.toml")
        .assert()
        .failure()
        .stderr(predicate::str::contains("invalid named colour"));
}

#[test]
fn test_invalid_colour_rgb() {
    Command::new(get_binary_location())
        .arg("-C")
        .arg("./tests/invalid_configs/invalid_colour_rgb.toml")
        .assert()
        .failure()
        .stderr(predicate::str::contains("invalid RGB"));
}

#[test]
fn test_invalid_colour_rgb_2() {
    Command::new(get_binary_location())
        .arg("-C")
        .arg("./tests/invalid_configs/invalid_colour_rgb_2.toml")
        .assert()
        .failure()
        .stderr(predicate::str::contains("invalid RGB"));
}

#[test]
fn test_invalid_colour_string() {
    Command::new(get_binary_location())
        .arg("-C")
        .arg("./tests/invalid_configs/invalid_colour_string.toml")
        .assert()
        .failure()
        .stderr(predicate::str::contains("invalid named colour"));
}

#[test]
fn test_lone_default_widget_count() {
    Command::new(get_binary_location())
        .arg("-C")
        .arg("./tests/invalid_configs/lone_default_widget_count.toml")
        .assert()
        .failure()
        .stderr(predicate::str::contains("it must be used with"));
}

#[test]
fn test_invalid_default_widget_count() {
    Command::new(get_binary_location())
        .arg("-C")
        .arg("./tests/invalid_configs/invalid_default_widget_count.toml")
        .assert()
        .failure()
        .stderr(predicate::str::contains("invalid number"));
}
