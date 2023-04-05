//! Based on [heim's implementation](https://github.com/heim-rs/heim/blob/master/heim-disk/src/sys/macos/counters.rs).

use std::ffi::OsStr;

use anyhow::bail;

use super::io_kit::{self, get_dict, get_disks, get_i64, get_string};

#[derive(Debug, Default)]
pub struct IoCounters {
    name: String,
    read_bytes: u64,
    write_bytes: u64,
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

fn get_device_io(device: io_kit::IoObject) -> anyhow::Result<IoCounters> {
    let parent = device.service_parent()?;

    if parent.conforms_to_block_storage_driver() {
        let disk_props = device.properties()?;
        let parent_props = parent.properties()?;

        let name = get_string(&disk_props, "BSD Name")?;
        let stats = get_dict(&parent_props, "Statistics")?;

        let read_bytes = get_i64(&stats, "Bytes (Read)")? as u64;
        let write_bytes = get_i64(&stats, "Bytes (Write)")? as u64;

        // let read_count = stats.get_i64("Operations (Read)")? as u64;
        // let write_count = stats.get_i64("Operations (Write)")? as u64;

        Ok(IoCounters {
            name,
            read_bytes,
            write_bytes,
        })
    } else {
        bail!("{parent:?}, the parent of {device:?} does not conform to IOBlockStorageDriver")
    }
}

/// Returns an iterator of disk I/O stats. Pulls data through IOKit.
pub fn io_stats() -> anyhow::Result<Vec<anyhow::Result<IoCounters>>> {
    Ok(get_disks()?.map(get_device_io).collect())
}
