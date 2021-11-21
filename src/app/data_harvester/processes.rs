//! Data collection for processes.
//!
//! For Linux, this is handled by a custom set of functions.
//! For Windows and macOS, this is handled by sysinfo.

cfg_if::cfg_if! {
    if #[cfg(target_os = "linux")] {
        pub mod linux;
        pub use self::linux::*;
    } else if #[cfg(target_os = "macos")] {
        pub mod macos;
        pub use self::macos::*;
    } else if #[cfg(target_os = "windows")] {
        pub mod windows;
        pub use self::windows::*;
    }
}

cfg_if::cfg_if! {
    if #[cfg(target_family = "unix")] {
        pub mod unix;
        pub use self::unix::*;
    }
}

use std::borrow::Cow;

use crate::Pid;

#[derive(Debug, Clone, Default)]
pub struct ProcessHarvest {
    pub pid: Pid,
    pub parent_pid: Option<Pid>,
    pub cpu_usage_percent: f64,
    pub mem_usage_percent: f64,
    pub mem_usage_bytes: u64,
    // pub rss_kb: u64,
    // pub virt_kb: u64,
    pub name: String,
    pub command: String,
    pub read_bytes_per_sec: u64,
    pub write_bytes_per_sec: u64,
    pub total_read_bytes: u64,
    pub total_write_bytes: u64,
    pub process_state: String,
    pub process_state_char: char,

    /// This is the effective user ID. This is only used on Unix platforms.
    #[cfg(target_family = "unix")]
    pub uid: libc::uid_t,

    /// This is the process' user. This is only used on Unix platforms.
    #[cfg(target_family = "unix")]
    pub user: Cow<'static, str>,
}

impl ProcessHarvest {
    pub(crate) fn add(&mut self, rhs: &ProcessHarvest) {
        self.cpu_usage_percent += rhs.cpu_usage_percent;
        self.mem_usage_bytes += rhs.mem_usage_bytes;
        self.mem_usage_percent += rhs.mem_usage_percent;
        self.read_bytes_per_sec += rhs.read_bytes_per_sec;
        self.write_bytes_per_sec += rhs.write_bytes_per_sec;
        self.total_read_bytes += rhs.total_read_bytes;
        self.total_write_bytes += rhs.total_write_bytes;
    }
}
