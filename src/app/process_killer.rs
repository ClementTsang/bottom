/// This file is meant to house (OS specific) implementations on how to kill processes.
use std::process::Command;

/// Kills a process, given a PID.
pub fn kill_process_given_pid(pid : i64) -> crate::utils::error::Result<()> {
	if cfg!(target_os = "linux") {
		// Linux
		Command::new("kill").arg(pid.to_string()).output()?;
	}
	else if cfg!(target_os = "windows") {
		// Windows
		debug!("Sorry, Windows support is not implemented yet!");
	}
	else if cfg!(target_os = "macos") {
		// TODO: macOS
		debug!("Sorry, macOS support is not implemented yet!");
	}
	else {
		// TODO: Others?
		debug!("Sorry, other support this is not implemented yet!");
	}

	Ok(())
}
