//! Extract GPU process information on Linux.

use std::{os::fd::BorrowedFd, path::Path};

use hashbrown::HashSet;
use rustix::fs::{Mode, OFlags};

use crate::collection::processes::Pid;

fn is_drm_fd(fd: &BorrowedFd<'_>) -> bool {
    true
}

/// Get fdinfo for a process given the PID.
///
/// Based on the method from nvtop [here](https://github.com/Syllo/nvtop/blob/339ee0b10a64ec51f43d27357b0068a40f16e9e4/src/extract_processinfo_fdinfo.c#L101).
pub(crate) fn get_fdinfo(pid: Pid, seen_fds: &mut HashSet<u32>) {
    let fdinfo_path = format!("/proc/{pid}/fdinfo");
    let fdinfo_path = Path::new(&fdinfo_path);

    let Ok(fd_entries) = std::fs::read_dir(fdinfo_path) else {
        return;
    };

    for fd_entry in fd_entries.flatten() {
        let path = fd_entry.path();

        if !path.is_file() {
            continue;
        }

        if !(path.to_string_lossy().chars().all(|c| c.is_ascii_digit())) {
            continue;
        }

        let Ok(fd) = rustix::fs::openat(
            pid_path.as_path(),
            OFlags::PATH | OFlags::DIRECTORY | OFlags::CLOEXEC,
            Mode::empty(),
        ) else {
            continue;
        };
    }
}
