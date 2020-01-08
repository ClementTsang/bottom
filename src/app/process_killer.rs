/// This file is meant to house (OS specific) implementations on how to kill processes.
use crate::utils::error::BottomError;
use std::process::Command;

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
pub fn kill_process_given_pid(pid: u32) -> crate::utils::error::Result<()> {
	if cfg!(target_os = "linux") {
		Command::new("kill").arg(pid.to_string()).output()?;
	} else if cfg!(target_os = "windows") {
		#[cfg(target_os = "windows")]
		{
			let process = Process::open(pid as DWORD)?;
			process.kill()?;
		}
	} else if cfg!(target_os = "macos") {
		// TODO: macOS
		return Err(BottomError::GenericError {
			message: "Sorry, macOS support is not implemented yet!".to_string(),
		});
	} else {
		return Err(BottomError::GenericError {
			message:
				"Sorry, support operating systems outside the main three are not implemented yet!"
					.to_string(),
		});
	}

	Ok(())
}
