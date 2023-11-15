//! This file is meant to house (OS specific) implementations on how to kill processes.

#[cfg(target_os = "windows")]
use windows::Win32::{
    Foundation::{CloseHandle, HANDLE},
    System::Threading::{
        OpenProcess, TerminateProcess, PROCESS_QUERY_INFORMATION, PROCESS_TERMINATE,
    },
};

#[cfg(target_family = "unix")]
use crate::utils::error::BottomError;
use crate::Pid;

/// Based from [this SO answer](https://stackoverflow.com/a/55231715).
#[cfg(target_os = "windows")]
struct Process(HANDLE);

#[cfg(target_os = "windows")]
impl Process {
    fn open(pid: u32) -> Result<Process, String> {
        // SAFETY: Windows API call, tread carefully with the args.
        match unsafe { OpenProcess(PROCESS_QUERY_INFORMATION | PROCESS_TERMINATE, false, pid) } {
            Ok(process) => Ok(Process(process)),
            Err(_) => Err("process may have already been terminated.".to_string()),
        }
    }

    fn kill(self) -> Result<(), String> {
        // SAFETY: Windows API call, this is safe as we are passing in the handle.
        let result = unsafe { TerminateProcess(self.0, 1) };
        if result.is_err() {
            return Err("process may have already been terminated.".to_string());
        }

        Ok(())
    }
}

#[cfg(target_os = "windows")]
impl Drop for Process {
    fn drop(&mut self) {
        // SAFETY: Windows API call, this is safe as we are passing in the handle.
        unsafe {
            let _ = CloseHandle(self.0);
        }
    }
}

/// Kills a process, given a PID, for windows.
#[cfg(target_os = "windows")]
pub fn kill_process_given_pid(pid: Pid) -> crate::utils::error::Result<()> {
    {
        let process = Process::open(pid as u32)?;
        process.kill()?;
    }

    Ok(())
}

/// Kills a process, given a PID, for unix.
#[cfg(target_family = "unix")]
pub fn kill_process_given_pid(pid: Pid, signal: usize) -> crate::utils::error::Result<()> {
    // SAFETY: the signal should be valid, and we act properly on an error (exit code not 0).
    let output = unsafe { libc::kill(pid, signal as i32) };
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
                "Error code {err_code} - {err}"
            )))
        } else {
            Err(BottomError::GenericError(format!("Error code ??? - {err}")))
        };
    }

    Ok(())
}
