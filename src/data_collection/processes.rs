//! Data collection for processes.
//!
//! For Linux, this is handled by a custom set of functions.
//! For Windows, macOS, FreeBSD, Android, and Linux, this is handled by sysinfo.

use cfg_if::cfg_if;
use std::{borrow::Cow, time::Duration};

use super::DataCollector;

use crate::{utils::error, Pid};

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

#[derive(Debug, Clone, Default)]
pub struct ProcessHarvest {
    /// The pid of the process.
    pub pid: Pid,

    /// The parent PID of the process. A `parent_pid` of 0 is usually the root.
    pub parent_pid: Option<Pid>,

    /// CPU usage as a percentage.
    pub cpu_usage_percent: f32,

    /// Memory usage as a percentage.
    pub mem_usage_percent: f32,

    /// Memory usage as bytes.
    pub mem_usage_bytes: u64,

    /// The name of the process.
    pub name: String,

    /// The exact command for the process.
    pub command: String,

    /// Bytes read per second.
    pub read_bytes_per_sec: u64,

    /// Bytes written per second.
    pub write_bytes_per_sec: u64,

    /// The total number of bytes read by the process.
    pub total_read_bytes: u64,

    /// The total number of bytes written by the process.
    pub total_write_bytes: u64,

    /// The current state of the process (e.g. zombie, asleep).
    pub process_state: (String, char),

    /// Cumulative total CPU time used.
    pub time: Duration,

    /// This is the *effective* user ID of the process. This is only used on Unix platforms.
    #[cfg(target_family = "unix")]
    pub uid: Option<libc::uid_t>,

    /// This is the process' user.
    pub user: Cow<'static, str>,

    /// Gpu memory usage as bytes.
    #[cfg(feature = "gpu")]
    pub gpu_mem: u64,

    /// Gpu memory usage as percentage.
    #[cfg(feature = "gpu")]
    pub gpu_mem_percent: f32,

    /// Gpu utilization as a percentage.
    #[cfg(feature = "gpu")]
    pub gpu_util: u32,
    // TODO: Additional fields
    // pub rss_kb: u64,
    // pub virt_kb: u64,
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
        self.time += rhs.time;
        #[cfg(feature = "gpu")]
        {
            self.gpu_mem += rhs.gpu_mem;
            self.gpu_util += rhs.gpu_util;
            self.gpu_mem_percent += rhs.gpu_mem_percent;
        }
    }
}

impl DataCollector {
    pub(crate) fn get_processes(&mut self) -> error::Result<Vec<ProcessHarvest>> {
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
                Err(error::BottomError::GenericError("Unsupported OS".to_string()))
            }
        }
    }
}
