/// This file is meant to house (OS specific) implementations on how to kill processes.
use std::process::Command;

// TODO: Make it update process list on freeze.

// Copied from SO: https://stackoverflow.com/a/55231715
#[cfg(target_os = "windows")]
use winapi::{
	shared::{minwindef::DWORD, ntdef::HANDLE},
	um::{
		processthreadsapi::{OpenProcess, TerminateProcess},
		winnt::{PROCESS_QUERY_INFORMATION, PROCESS_TERMINATE},
	},
};

#[cfg(target_os = "windows")]
struct Process(HANDLE);

#[cfg(target_os = "windows")]
impl Process {
	fn open(pid: DWORD) -> Result<Process, String> {
		let pc = unsafe { OpenProcess(PROCESS_QUERY_INFORMATION | PROCESS_TERMINATE, 0, pid) };
		if pc.is_null() {
			return Err("!OpenProcess".to_string());
		}
		Ok(Process(pc))
	}

	fn kill(self) -> Result<(), String> {
		unsafe { TerminateProcess(self.0, 1) };
		Ok(())
	}
}

/// Kills a process, given a PID.
pub fn kill_process_given_pid(pid: u64) -> crate::utils::error::Result<()> {
	if cfg!(target_os = "linux") {
		// Linux
		Command::new("kill").arg(pid.to_string()).output()?;
	} else if cfg!(target_os = "windows") {
		#[cfg(target_os = "windows")]
		let process = Process::open(pid as DWORD)?;
		#[cfg(target_os = "windows")]
		process.kill()?;
	} else if cfg!(target_os = "macos") {
		// TODO: macOS
		// See how sysinfo does it... https://docs.rs/sysinfo/0.9.5/sysinfo/trait.ProcessExt.html
		debug!("Sorry, macOS support is not implemented yet!");
	} else {
		// TODO: Others?
		debug!("Sorry, other support this is not implemented yet!");
	}

	Ok(())
}
