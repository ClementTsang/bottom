use assert_cmd::prelude::*; // Add methods on commands
use predicates::prelude::*;
use std::process::Command; // Run programs // Used for writing assertions

//======================RATES======================//
#[test]
fn valid_rate_argument() {
}

#[test]
fn test_small_rate() -> Result<(), Box<dyn std::error::Error>> {
	Command::new("./target/debug/rustop")
		.arg("-r")
		.arg("249")
		.assert()
		.failure()
		.stderr(predicate::str::contains("rate to be greater than 250"));
	Ok(())
}

#[test]
fn test_large_rate() -> Result<(), Box<dyn std::error::Error>> {
	Command::new("./target/debug/rustop")
		.arg("-r")
		.arg("18446744073709551616")
		.assert()
		.failure()
		.stderr(predicate::str::contains("rate to be less than unsigned INT_MAX."));
	Ok(())
}

#[test]
fn test_negative_rate() -> Result<(), Box<dyn std::error::Error>> {
	// This test should auto fail due to how clap works
	Command::new("./target/debug/rustop")
		.arg("-r")
		.arg("-1000")
		.assert()
		.failure()
		.stderr(predicate::str::contains("wasn't expected, or isn't valid in this context"));

	Ok(())
}

#[test]
fn test_invalid_rate() -> Result<(), Box<dyn std::error::Error>> {
	Command::new("./target/debug/rustop")
		.arg("-r")
		.arg("100-1000")
		.assert()
		.failure()
		.stderr(predicate::str::contains("invalid digit"));

	Ok(())
}
