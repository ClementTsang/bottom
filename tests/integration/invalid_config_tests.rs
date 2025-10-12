//! These tests are for testing some invalid config-file-specific options.

use assert_cmd::prelude::*;
use predicates::prelude::*;

use crate::util::btm_command;

#[test]
fn test_toml_mismatch_type() {
    btm_command(&["-C", "./tests/invalid_configs/toml_mismatch_type.toml"])
        .assert()
        .failure()
        .stderr(predicate::str::contains("invalid type"));
}

#[test]
fn test_empty_layout() {
    btm_command(&["-C", "./tests/invalid_configs/empty_layout.toml"])
        .assert()
        .failure()
        .stderr(predicate::str::contains("at least one widget"));
}

#[test]
fn test_invalid_layout_widget_type() {
    btm_command(&[
        "-C",
        "./tests/invalid_configs/invalid_layout_widget_type.toml",
    ])
    .assert()
    .failure()
    .stderr(predicate::str::contains("invalid widget name"));
}

/// This test isn't really needed as this is technically covered by TOML spec.
/// However, I feel like it's worth checking anyways - not like it takes long.
#[test]
fn test_duplicate_temp_type() {
    btm_command(&["-C", "./tests/invalid_configs/duplicate_temp_type.toml"])
        .assert()
        .failure()
        .stderr(predicate::str::contains("duplicate key"));
}

/// Checks for if a hex is valid
#[test]
fn test_invalid_colour_hex() {
    btm_command(&["-C", "./tests/invalid_configs/invalid_colour_hex.toml"])
        .assert()
        .failure()
        .stderr(predicate::str::contains("invalid hex color"));
}

/// Checks for if a hex is too long
#[test]
fn test_invalid_colour_hex_2() {
    btm_command(&["-C", "./tests/invalid_configs/invalid_colour_hex_2.toml"])
        .assert()
        .failure()
        .stderr(predicate::str::contains("invalid hex color"));
}

/// Checks unicode hex because the way we originally did it could cause char
/// boundary errors!
#[test]
fn test_invalid_colour_hex_3() {
    btm_command(&["-C", "./tests/invalid_configs/invalid_colour_hex_3.toml"])
        .assert()
        .failure()
        .stderr(predicate::str::contains("invalid hex color"));
}

#[test]
fn test_invalid_colour_name() {
    btm_command(&["-C", "./tests/invalid_configs/invalid_colour_name.toml"])
        .assert()
        .failure()
        .stderr(predicate::str::contains("invalid named color"));
}

#[test]
fn test_invalid_colour_rgb() {
    btm_command(&["-C", "./tests/invalid_configs/invalid_colour_rgb.toml"])
        .assert()
        .failure()
        .stderr(predicate::str::contains("invalid RGB"));
}

#[test]
fn test_invalid_colour_rgb_2() {
    btm_command(&["-C", "./tests/invalid_configs/invalid_colour_rgb_2.toml"])
        .assert()
        .failure()
        .stderr(predicate::str::contains("invalid RGB"));
}

#[test]
fn test_invalid_colour_string() {
    btm_command(&["-C", "./tests/invalid_configs/invalid_colour_string.toml"])
        .assert()
        .failure()
        .stderr(predicate::str::contains("invalid named color"));
}

#[test]
fn test_lone_default_widget_count() {
    btm_command(&[
        "-C",
        "./tests/invalid_configs/lone_default_widget_count.toml",
    ])
    .assert()
    .failure()
    .stderr(predicate::str::contains("it must be used with"));
}

#[test]
fn test_invalid_default_widget_count() {
    btm_command(&[
        "-C",
        "./tests/invalid_configs/invalid_default_widget_count.toml",
    ])
    .assert()
    .failure()
    .stderr(predicate::str::contains("integer number overflowed"));
}

#[test]
fn test_invalid_process_column() {
    btm_command(&["-C", "./tests/invalid_configs/invalid_process_column.toml"])
        .assert()
        .failure()
        .stderr(predicate::str::contains("doesn't match"));
}

#[test]
fn test_invalid_disk_column() {
    btm_command(&["-C", "./tests/invalid_configs/invalid_disk_column.toml"])
        .assert()
        .failure()
        .stderr(predicate::str::contains("doesn't match"));
}
