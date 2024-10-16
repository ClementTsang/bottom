use std::{borrow::Cow, time::Duration};

use crate::new_data_collection::sources::Pid;

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

    /// Cumulative process uptime.
    pub time: Duration,

    /// This is the *effective* user ID of the process. This is only used on
    /// Unix platforms.
    #[cfg(target_family = "unix")]
    pub uid: Option<libc::uid_t>,

    /// This is the process' user.
    pub user: Cow<'static, str>,

    /// GPU memory usage as bytes.
    #[cfg(feature = "gpu")]
    pub gpu_mem: u64,

    /// GPU memory usage as percentage.
    #[cfg(feature = "gpu")]
    pub gpu_mem_percent: f32,

    /// GPU utilization as a percentage.
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
        self.time = self.time.max(rhs.time);
        #[cfg(feature = "gpu")]
        {
            self.gpu_mem += rhs.gpu_mem;
            self.gpu_util += rhs.gpu_util;
            self.gpu_mem_percent += rhs.gpu_mem_percent;
        }
    }
}
