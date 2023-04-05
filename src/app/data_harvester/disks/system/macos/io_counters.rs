//! Based on [heim's implementation](https://github.com/heim-rs/heim/blob/master/heim-disk/src/sys/macos/counters.rs).

use std::ffi::OsStr;

#[derive(Debug, Default)]
pub struct IoCounters {
    name: String,

    read_count: u64,
    read_bytes: u64,
    read_time_secs: u64,
    write_count: u64,
    write_bytes: u64,
    write_time_secs: u64,
}

impl IoCounters {
    pub(crate) fn device_name(&self) -> &OsStr {
        OsStr::new(&self.name)
    }

    pub(crate) fn read_bytes(&self) -> u64 {
        self.read_bytes
    }

    pub(crate) fn write_bytes(&self) -> u64 {
        self.write_bytes
    }
}

/// Returns an iterator of disk I/O stats. Pulls data from `/proc/diskstats`.
pub fn io_stats() -> anyhow::Result<Vec<anyhow::Result<IoCounters>>> {
    Ok(vec![])
}
