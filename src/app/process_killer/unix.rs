use crate::utils::error::BottomError;
use crate::Pid;

/// Kills a process, given a PID, for unix.
pub(crate) fn kill_process_given_pid(pid: Pid, signal: usize) -> crate::utils::error::Result<()> {
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
