//! These tests are mostly here just to ensure that invalid results will be caught when passing arguments.
use assert_cmd::prelude::*;
use predicates::prelude::*;

mod util;
use util::*;

#[test]
fn test_small_rate() {
    btm_command()
        .arg("-C")
        .arg("./tests/empty_config.toml")
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
    btm_command()
        .arg("-C")
        .arg("./tests/empty_config.toml")
        .arg("-t")
        .arg("18446744073709551616")
        .assert()
        .failure()
        .stderr(predicate::str::contains("could not parse"));
}

#[test]
fn test_small_default_time() {
    btm_command()
        .arg("-C")
        .arg("./tests/empty_config.toml")
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
    btm_command()
        .arg("-C")
        .arg("./tests/empty_config.toml")
        .arg("-d")
        .arg("18446744073709551616")
        .assert()
        .failure()
        .stderr(predicate::str::contains("could not parse"));
}

#[test]
fn test_small_delta_time() {
    btm_command()
        .arg("-C")
        .arg("./tests/empty_config.toml")
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
    btm_command()
        .arg("-C")
        .arg("./tests/empty_config.toml")
        .arg("-r")
        .arg("18446744073709551616")
        .assert()
        .failure()
        .stderr(predicate::str::contains("could not parse"));
}

#[test]
fn test_negative_rate() {
    // This test should auto fail due to how clap works
    btm_command()
        .arg("-C")
        .arg("./tests/empty_config.toml")
        .arg("-r")
        .arg("-1000")
        .assert()
        .failure()
        .stderr(predicate::str::contains("unexpected argument"));
}

#[test]
fn test_invalid_rate() {
    btm_command()
        .arg("-C")
        .arg("./tests/empty_config.toml")
        .arg("-r")
        .arg("100-1000")
        .assert()
        .failure()
        .stderr(predicate::str::contains("could not parse"));
}

#[test]
fn test_conflicting_temps() {
    btm_command()
        .arg("-C")
        .arg("./tests/empty_config.toml")
        .arg("-c")
        .arg("-f")
        .assert()
        .failure()
        .stderr(predicate::str::contains("cannot be used with"));
}

#[test]
fn test_invalid_default_widget_1() {
    btm_command()
        .arg("-C")
        .arg("./tests/empty_config.toml")
        .arg("--default_widget_type")
        .arg("fake_widget")
        .assert()
        .failure()
        .stderr(predicate::str::contains("invalid widget name"));
}

#[test]
fn test_invalid_default_widget_2() {
    btm_command()
        .arg("-C")
        .arg("./tests/empty_config.toml")
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
    btm_command()
        .arg("-C")
        .arg("./tests/empty_config.toml")
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
    btm_command()
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
    btm_command()
        .arg("--enable_gpu")
        .assert()
        .failure()
        .stderr(predicate::str::contains(
            "unexpected argument '--enable_gpu' found",
        ));
}
