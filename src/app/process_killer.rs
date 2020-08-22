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
use crate::utils::error::BottomError;

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

/// Kills a process, given a PID.
pub fn kill_process_given_pid(pid: u32) -> crate::utils::error::Result<()> {
    if cfg!(target_os = "linux") || cfg!(target_os = "macos") {
        #[cfg(any(target_os = "linux", target_os = "macos"))]
        {
            let output = unsafe { libc::kill(pid as i32, libc::SIGTERM) };
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
        }
    } else if cfg!(target_os = "windows") {
        #[cfg(target_os = "windows")]
        {
            let process = Process::open(pid as DWORD)?;
            process.kill()?;
        }
    } else {
        return Err(BottomError::GenericError(
            "Sorry, support operating systems outside the main three are not implemented yet!"
                .to_string(),
        ));
    }

    Ok(())
}
