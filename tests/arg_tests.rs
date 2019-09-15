use std::process::Command;  // Run programs
use assert_cmd::prelude::*; // Add methods on commands
use predicates::prelude::*; // Used for writing assertions

#[test]
fn test_small_rate -> Result<(), Box<std::error::Error>> {
    let mut cmd = Command::main_binary()?;
    cmd.arg("-r")
        .arg("249");
    cmd.assert()
        .failure()
        .stderr(predicate::str::contains("rate"));

    Ok(())
}

#[test]
fn test_negative_rate -> Result<(), Box<std::error::Error>> {
    // This test should auto fail due to how clap works
    let mut cmd = Command::main_binary()?;
    cmd.arg("-r")
        .arg("-1000");
    cmd.assert()
        .failure()
        .stderr(predicate::str::contains("valid"));

    Ok(())
}

#[test]
fn test_invalid_rate -> Result<(), Box<std::error::Error>> {
    let mut cmd = Command::main_binary()?;
    cmd.arg("-r")
        .arg("1000 - 100");
    cmd.assert()
        .failure()
        .stderr(predicate::str::contains("digit"));

    Ok(())
}