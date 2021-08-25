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

use crate::Pid;

// TODO: Add value so we know if it's sorted ascending or descending by default?
#[derive(Clone, PartialEq, Eq, Hash, Debug)]
pub enum ProcessSorting {
    CpuPercent,
    Mem,
    MemPercent,
    Pid,
    ProcessName,
    Command,
    ReadPerSecond,
    WritePerSecond,
    TotalRead,
    TotalWrite,
    State,
    User,
    Count,
}

impl std::fmt::Display for ProcessSorting {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "{}",
            match &self {
                ProcessSorting::CpuPercent => "CPU%",
                ProcessSorting::MemPercent => "Mem%",
                ProcessSorting::Mem => "Mem",
                ProcessSorting::ReadPerSecond => "R/s",
                ProcessSorting::WritePerSecond => "W/s",
                ProcessSorting::TotalRead => "T.Read",
                ProcessSorting::TotalWrite => "T.Write",
                ProcessSorting::State => "State",
                ProcessSorting::ProcessName => "Name",
                ProcessSorting::Command => "Command",
                ProcessSorting::Pid => "PID",
                ProcessSorting::Count => "Count",
                ProcessSorting::User => "User",
            }
        )
    }
}

impl Default for ProcessSorting {
    fn default() -> Self {
        ProcessSorting::CpuPercent
    }
}

#[derive(Debug, Clone, Default)]
pub struct ProcessHarvest {
    pub pid: Pid,
    pub parent_pid: Option<Pid>, // Remember, parent_pid 0 is root...
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

    /// This is the *effective* user ID.
    #[cfg(target_family = "unix")]
    pub uid: Option<libc::uid_t>,
}
