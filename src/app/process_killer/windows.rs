// Copied from SO: https://stackoverflow.com/a/55231715
use winapi::{
    shared::{minwindef::DWORD, ntdef::HANDLE},
    um::{
        processthreadsapi::{OpenProcess, TerminateProcess},
        winnt::{PROCESS_QUERY_INFORMATION, PROCESS_TERMINATE},
    },
};

pub(crate) struct Process(HANDLE);

impl Process {
    pub(crate) fn open(pid: DWORD) -> Result<Process, String> {
        let pc = unsafe { OpenProcess(PROCESS_QUERY_INFORMATION | PROCESS_TERMINATE, 0, pid) };
        if pc.is_null() {
            return Err("OpenProcess".to_string());
        }
        Ok(Process(pc))
    }

    pub(crate) fn kill(self) -> Result<(), String> {
        let result = unsafe { TerminateProcess(self.0, 1) };
        if result == 0 {
            return Err("Failed to kill process".to_string());
        }

        Ok(())
    }
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
