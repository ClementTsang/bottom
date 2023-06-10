//! Data collection for processes.
//!
//! For Linux, this is handled by a custom set of functions.
//! For Windows, macOS, FreeBSD, Android, and Linux, this is handled by sysinfo.

cfg_if::cfg_if! {
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
    }
}

cfg_if::cfg_if! {
    if #[cfg(target_family = "unix")] {
        pub mod unix;
        pub use self::unix::*;
    }
}

use std::{borrow::Cow, time::Duration};

use sysinfo::SystemExt;

use crate::{utils::error, Pid};

use super::DataCollector;

#[derive(Debug, Clone, Default)]
pub struct ProcessHarvest {
    /// The pid of the process.
    pub pid: Pid,

    /// The parent PID of the process. A `parent_pid` of 0 is usually the root.
    pub parent_pid: Option<Pid>,

    /// CPU usage as a percentage.
    pub cpu_usage_percent: f64,

    /// Memory usage as a percentage.
    pub mem_usage_percent: f64,

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
    }
}

impl DataCollector {
    pub(crate) fn get_processes(&mut self) -> error::Result<Vec<ProcessHarvest>> {
        let total_memory = if let Some(memory) = &self.data.memory {
            memory.total_bytes
        } else {
            self.sys.total_memory()
        };

        #[cfg(target_os = "linux")]
        {
            let current_instant = self.data.collection_time;

            let prev_proc = PrevProc {
                prev_idle: &mut self.prev_idle,
                prev_non_idle: &mut self.prev_non_idle,
            };

            let proc_harvest_options = ProcHarvestOptions {
                use_current_cpu_total: self.use_current_cpu_total,
                unnormalized_cpu: self.unnormalized_cpu,
            };

            let time_diff = current_instant
                .duration_since(self.last_collection_time)
                .as_secs();

            get_process_data(
                &self.sys,
                prev_proc,
                &mut self.pid_mapping,
                proc_harvest_options,
                time_diff,
                total_memory,
                &mut self.user_table,
            )
        }
        #[cfg(not(target_os = "linux"))]
        {
            get_process_data(
                &self.sys,
                self.use_current_cpu_total,
                self.unnormalized_cpu,
                total_memory,
                #[cfg(target_family = "unix")]
                &mut self.user_table,
            )
        }
    }
}
