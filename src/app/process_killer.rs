/// This file is meant to house (OS specific) implementations on how to kill processes.
use std::process::Command;

/// Kills a process, given a PID.
pub fn kill_process_given_pid(pid : u64) -> crate::utils::error::Result<()> {
	if cfg!(target_os = "linux") {
		// Linux
		Command::new("kill").arg(pid.to_string()).output()?;
	}
	else if cfg!(target_os = "windows") {
		// Windows
		// See how sysinfo does it... https://docs.rs/sysinfo/0.9.5/sysinfo/trait.ProcessExt.html
		debug!("Sorry, Windows support is not implemented yet!");
	}
	else if cfg!(target_os = "macos") {
		// TODO: macOS
		// See how sysinfo does it... https://docs.rs/sysinfo/0.9.5/sysinfo/trait.ProcessExt.html
		debug!("Sorry, macOS support is not implemented yet!");
	}
	else {
		// TODO: Others?
		debug!("Sorry, other support this is not implemented yet!");
	}

	Ok(())
}
