/// This file is meant to house (OS specific) implementations on how to kill processes.
use std::process::Command;
use std::ptr::null_mut;
use winapi::{
	shared::{minwindef::DWORD, ntdef::HANDLE},
	um::{
		processthreadsapi::{OpenProcess, TerminateProcess},
		winnt::{PROCESS_QUERY_INFORMATION, PROCESS_TERMINATE},
	},
};

// Copied from SO: https://stackoverflow.com/a/55231715
struct Process(HANDLE);
impl Process {
	fn open(pid : DWORD) -> Result<Process, String> {
		let pc = unsafe { OpenProcess(PROCESS_QUERY_INFORMATION | PROCESS_TERMINATE, 0, pid) };
		if pc == null_mut() {
			return Err("!OpenProcess".to_string());
		}
		Ok(Process(pc))
	}

	fn kill(self) -> Result<(), String> {
		unsafe { TerminateProcess(self.0, 1) };
		Ok(())
	}
}
impl Drop for Process {
	fn drop(&mut self) {
		unsafe { winapi::um::handleapi::CloseHandle(self.0) };
	}
}

/// Kills a process, given a PID.
pub fn kill_process_given_pid(pid : u64) -> crate::utils::error::Result<()> {
	if cfg!(target_os = "linux") {
		// Linux
		Command::new("kill").arg(pid.to_string()).output()?;
	}
	else if cfg!(target_os = "windows") {
		let process = Process::open(pid as DWORD)?;
		process.kill()?;
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
