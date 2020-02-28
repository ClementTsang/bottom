use assert_cmd::prelude::*;
use predicates::prelude::*;
use std::process::Command;

// These tests are mostly here just to ensure that invalid results will be caught when passing arguments...

// TODO: [TEST] Allow for release testing.

//======================RATES======================//

fn get_os_binary_loc() -> String {
	if cfg!(target_os = "linux") {
		"./target/x86_64-unknown-linux-gnu/debug/btm".to_string()
	}
	else if cfg!(target_os = "windows") {
		"./target/x86_64-pc-windows-msvc/debug/btm".to_string()
	}
	else if cfg!(target_os = "macos") {
		"./target/x86_64-apple-darwin/debug/btm".to_string()
	}
	else {
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
		.stderr(predicate::str::contains("rate to be greater than 250"));
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
			"rate to be less than unsigned INT_MAX.",
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
