//! These tests are mostly here just to ensure that invalid results will be caught when passing arguments.

use assert_cmd::prelude::*;
use predicates::prelude::*;

use crate::util::{btm_command, no_cfg_btm_command};

#[test]
fn test_small_rate() {
    btm_command(&["-C", "./tests/valid_configs/empty_config.toml"])
        .arg("-r")
        .arg("249")
        .assert()
        .failure()
        .stderr(predicate::str::contains(
            "set your update rate to be at least 250 ms.",
        ));
}

#[test]
fn test_large_default_time() {
    no_cfg_btm_command()
        .arg("-t")
        .arg("18446744073709551616")
        .assert()
        .failure()
        .stderr(predicate::str::contains("could not parse"));
}

#[test]
fn test_small_default_time() {
    no_cfg_btm_command()
        .arg("-t")
        .arg("900")
        .assert()
        .failure()
        .stderr(predicate::str::contains(
            "set your default value to be at least",
        ));
}

#[test]
fn test_large_delta_time() {
    no_cfg_btm_command()
        .arg("-d")
        .arg("18446744073709551616")
        .assert()
        .failure()
        .stderr(predicate::str::contains("could not parse"));
}

#[test]
fn test_small_delta_time() {
    no_cfg_btm_command()
        .arg("-d")
        .arg("900")
        .assert()
        .failure()
        .stderr(predicate::str::contains(
            "set your time delta to be at least",
        ));
}

#[test]
fn test_large_rate() {
    no_cfg_btm_command()
        .arg("-r")
        .arg("18446744073709551616")
        .assert()
        .failure()
        .stderr(predicate::str::contains("could not parse"));
}

#[test]
fn test_negative_rate() {
    // This test should auto fail due to how clap works
    no_cfg_btm_command()
        .arg("-r")
        .arg("-1000")
        .assert()
        .failure()
        .stderr(predicate::str::contains("unexpected argument"));
}

#[test]
fn test_invalid_rate() {
    no_cfg_btm_command()
        .arg("-r")
        .arg("100-1000")
        .assert()
        .failure()
        .stderr(predicate::str::contains("could not parse"));
}

#[test]
fn test_conflicting_temps() {
    no_cfg_btm_command()
        .arg("-c")
        .arg("-f")
        .assert()
        .failure()
        .stderr(predicate::str::contains("cannot be used with"));
}

#[test]
fn test_invalid_default_widget_1() {
    no_cfg_btm_command()
        .arg("--default_widget_type")
        .arg("fake_widget")
        .assert()
        .failure()
        .stderr(predicate::str::contains("invalid value"));
}

#[test]
fn test_invalid_default_widget_2() {
    no_cfg_btm_command()
        .arg("--default_widget_type")
        .arg("cpu")
        .arg("--default_widget_count")
        .arg("18446744073709551616")
        .assert()
        .failure()
        .stderr(predicate::str::contains(
            "set your widget count to be at most unsigned INT_MAX",
        ));
}

#[test]
fn test_missing_default_widget_type() {
    no_cfg_btm_command()
        .arg("--default_widget_count")
        .arg("3")
        .assert()
        .failure()
        .stderr(predicate::str::contains(
            "the following required arguments were not provided",
        ));
}

#[test]
#[cfg_attr(feature = "battery", ignore)]
fn test_battery_flag() {
    no_cfg_btm_command()
        .arg("--battery")
        .assert()
        .failure()
        .stderr(predicate::str::contains(
            "unexpected argument '--battery' found",
        ));
}

#[test]
#[cfg_attr(feature = "gpu", ignore)]
fn test_gpu_flag() {
    no_cfg_btm_command()
        .arg("--enable_gpu")
        .assert()
        .failure()
        .stderr(predicate::str::contains(
            "unexpected argument '--enable_gpu' found",
        ));
}
