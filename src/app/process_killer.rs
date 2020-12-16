// Copied from SO: https://stackoverflow.com/a/55231715
#[cfg(target_os = "windows")]
use winapi::{
    shared::{minwindef::DWORD, ntdef::HANDLE},
    um::{
        processthreadsapi::{OpenProcess, TerminateProcess},
        winnt::{PROCESS_QUERY_INFORMATION, PROCESS_TERMINATE},
    },
};

/// This file is meant to house (OS specific) implementations on how to kill processes.
#[cfg(target_family = "unix")]
use crate::utils::error::BottomError;
use crate::Pid;

#[cfg(target_os = "windows")]
struct Process(HANDLE);

#[cfg(target_os = "windows")]
impl Process {
    fn open(pid: DWORD) -> Result<Process, String> {
        let pc = unsafe { OpenProcess(PROCESS_QUERY_INFORMATION | PROCESS_TERMINATE, 0, pid) };
        if pc.is_null() {
            return Err("OpenProcess".to_string());
        }
        Ok(Process(pc))
    }

    fn kill(self) -> Result<(), String> {
        unsafe { TerminateProcess(self.0, 1) };
        Ok(())
    }
}

/// Kills a process, given a PID, for unix.
#[cfg(target_family = "unix")]
pub fn kill_process_given_pid(pid: Pid, signal: usize) -> crate::utils::error::Result<()> {
    let output = unsafe { libc::kill(pid as i32, signal as i32) };
    if output != 0 {
        // We had an error...
        let err_code = std::io::Error::last_os_error().raw_os_error();
        let err = match err_code {
            Some(libc::ESRCH) => "the target process did not exist.",
            Some(libc::EPERM) => "the calling process does not have the permissions to terminate the target process(es).",
            Some(libc::EINVAL) => "an invalid signal was specified.",
            _ => "Unknown error occurred."
        };

        return if let Some(err_code) = err_code {
            Err(BottomError::GenericError(format!(
                "Error code {} - {}",
                err_code, err,
            )))
        } else {
            Err(BottomError::GenericError(format!(
                "Error code ??? - {}",
                err,
            )))
        };
    }

    Ok(())
}

/// Kills a process, given a PID, for windows.
#[cfg(target_os = "windows")]
pub fn kill_process_given_pid(pid: Pid) -> crate::utils::error::Result<()> {
    {
        let process = Process::open(pid as DWORD)?;
        process.kill()?;
    }

    Ok(())
}
