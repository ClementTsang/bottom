//! Data collection for processes.
//!
//! For Linux, this is handled by a custom set of functions.
//! For Windows, macOS, FreeBSD, Android, and Linux, this is handled by sysinfo.

use cfg_if::cfg_if;
use sysinfo::ProcessStatus;

cfg_if! {
    if #[cfg(target_os = "linux")] {
        pub mod linux;
        pub use self::linux::*;
    } else if #[cfg(target_os = "macos")] {
        pub mod macos;
        pub(crate) use self::macos::*;
    } else if #[cfg(target_os = "windows")] {
        pub mod windows;
        pub use self::windows::*;
    } else if #[cfg(target_os = "freebsd")] {
        pub mod freebsd;
        pub(crate) use self::freebsd::*;
    } else if #[cfg(target_family = "unix")] {
        pub(crate) struct GenericProcessExt;
        impl UnixProcessExt for GenericProcessExt {}
    }
}

cfg_if! {
    if #[cfg(target_family = "unix")] {
        pub mod unix;
        pub use self::unix::*;
    }
}

use std::{sync::Arc, time::Duration};

use super::{DataCollector, error::CollectionResult};

cfg_if! {
    if #[cfg(target_family = "windows")] {
        /// A Windows process ID.
        pub type Pid = usize;
    } else if #[cfg(target_family = "unix")] {
        /// A UNIX process ID.
        pub type Pid = libc::pid_t;
    }
}

pub type Bytes = u64;

#[cfg(target_os = "linux")]
/// The process entry "type".
#[derive(Debug, Clone, Copy, Default)]
pub enum ProcessType {
    /// A regular user process.
    #[default]
    Regular,

    /// A kernel process.
    Kernel,

    /// A thread spawned by a regular user process.
    ProcessThread,
}

#[cfg(target_os = "linux")]
impl ProcessType {
    /// Returns `true` if this is a thread.
    pub fn is_thread(&self) -> bool {
        matches!(self, Self::ProcessThread)
    }

    /// Returns `true` if this is a kernel process.
    pub fn is_kernel(&self) -> bool {
        matches!(self, Self::Kernel)
    }
}

#[derive(Debug, Clone, Default)]
pub struct ProcessHarvest {
    /// The pid of the process.
    pub pid: Pid,

    /// The parent PID of the process. A `parent_pid` of 0 is usually the root.
    pub parent_pid: Option<Pid>,

    /// CPU usage as a percentage.
    pub cpu_usage_percent: f32,

    /// Memory usage as a percentage.
    ///
    /// TODO: Maybe calculate this on usage? Store the total mem along with the vector of results.
    pub mem_usage_percent: f32,

    /// Memory usage as bytes.
    pub mem_usage: Bytes,

    /// Virtual memory.
    pub virtual_mem: Bytes,

    /// The name of the process.
    pub name: String,

    /// The exact command for the process.
    pub command: String,

    /// Bytes read per second.
    pub read_per_sec: Bytes,

    /// Bytes written per second.
    pub write_per_sec: Bytes,

    /// The total number of bytes read by the process.
    pub total_read: Bytes,

    /// The total number of bytes written by the process.
    pub total_write: Bytes,

    /// The current state of the process (e.g. zombie, asleep).
    pub process_state: (&'static str, char),

    /// Cumulative process uptime.
    pub time: Duration,

    /// This is the *effective* user ID of the process. This is only used on
    /// Unix platforms.
    #[cfg(target_family = "unix")]
    pub uid: Option<libc::uid_t>,

    /// This is the process' user.
    pub user: Option<Arc<str>>,

    /// Gpu memory usage as bytes.
    #[cfg(feature = "gpu")]
    pub gpu_mem: u64,

    /// Gpu memory usage as percentage.
    ///
    /// TODO: Maybe calculate this on usage? Store the total GPU mem along with the vector of results.
    #[cfg(feature = "gpu")]
    pub gpu_mem_percent: f32,

    /// Gpu utilization as a percentage.
    #[cfg(feature = "gpu")]
    pub gpu_util: u32,

    /// The process entry "type".
    #[cfg(target_os = "linux")]
    pub process_type: ProcessType,
    // TODO: Additional fields
    // pub rss_kb: u64,
    // pub virt_kb: u64,
}

impl DataCollector {
    pub(crate) fn get_processes(&mut self) -> CollectionResult<Vec<ProcessHarvest>> {
        cfg_if! {
            if #[cfg(target_os = "linux")] {
                let time_diff = self.data.collection_time
                    .duration_since(self.last_collection_time)
                    .as_secs();

                linux_process_data(
                    self,
                    time_diff,
                )
            } else if #[cfg(any(target_os = "freebsd", target_os = "macos", target_os = "windows", target_os = "android", target_os = "ios"))] {
                sysinfo_process_data(self)
            } else {
                Err(crate::collection::error::CollectionError::Unsupported)
            }
        }
    }
}

/// Pulled from [`ProcessStatus::to_string`] to avoid an alloc.
pub(super) fn process_status_str(status: ProcessStatus) -> &'static str {
    cfg_if::cfg_if! {
        if #[cfg(target_os = "linux")] {
            match status {
                ProcessStatus::Idle => "Idle",
                ProcessStatus::Run => "Runnable",
                ProcessStatus::Sleep => "Sleeping",
                ProcessStatus::Stop => "Stopped",
                ProcessStatus::Zombie => "Zombie",
                ProcessStatus::Tracing => "Tracing",
                ProcessStatus::Dead => "Dead",
                ProcessStatus::Wakekill => "Wakekill",
                ProcessStatus::Waking => "Waking",
                ProcessStatus::Parked => "Parked",
                ProcessStatus::UninterruptibleDiskSleep => "UninterruptibleDiskSleep",
                _ => "Unknown",
            }
        } else if #[cfg(target_os = "windows")] {
            match status {
                ProcessStatus::Run => "Runnable",
                _ => "Unknown",
            }
        } else if #[cfg(target_os = "macos")] {
            match status {
                ProcessStatus::Idle => "Idle",
                ProcessStatus::Run => "Runnable",
                ProcessStatus::Sleep => "Sleeping",
                ProcessStatus::Stop => "Stopped",
                ProcessStatus::Zombie => "Zombie",
                _ => "Unknown",
            }
        } else if #[cfg(target_os = "freebsd")] {
            match status {
                ProcessStatus::Idle => "Idle",
                ProcessStatus::Run => "Runnable",
                ProcessStatus::Sleep => "Sleeping",
                ProcessStatus::Stop => "Stopped",
                ProcessStatus::Zombie => "Zombie",
                ProcessStatus::Dead => "Dead",
                ProcessStatus::LockBlocked => "LockBlocked",
                _ => "Unknown",
            }
        } else {
            "Unknown"
        }
    }
}
