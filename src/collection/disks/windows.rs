//! Disk stats via sysinfo.

mod bindings;

use bindings::*;
use itertools::Itertools;

use super::{DiskHarvest, keep_disk_entry};
use crate::collection::{DataCollector, disks::IoCounters};

/// Returns I/O stats.
pub(crate) fn io_stats() -> anyhow::Result<Vec<IoCounters>> {
    let volume_io = all_volume_io()?;

    Ok(volume_io
        .into_iter()
        .map_ok(|(performance, volume_name)| {
            let name = volume_name;
            let read_bytes = performance.BytesRead as u64;
            let write_bytes = performance.BytesWritten as u64;

            IoCounters::new(name, read_bytes, write_bytes)
        })
        .flatten()
        .collect::<Vec<_>>())
}

pub(crate) fn get_disk_usage(collector: &DataCollector) -> anyhow::Result<Vec<DiskHarvest>> {
    let disks = &collector.sys.disks;
    let disk_filter = &collector.filters.disk_filter;
    let mount_filter = &collector.filters.mount_filter;

    Ok(disks
        .iter()
        .filter_map(|disk| {
            let name = {
                let name = disk.name();

                if name.is_empty() {
                    "No Name".to_string()
                } else {
                    name.to_os_string()
                        .into_string()
                        .unwrap_or_else(|_| "Name Unavailable".to_string())
                }
            };

            let mount_point = disk
                .mount_point()
                .as_os_str()
                .to_os_string()
                .into_string()
                .unwrap_or_else(|_| "Mount Unavailable".to_string());

            let volume_name = volume_name_from_mount(&mount_point).ok();

            if keep_disk_entry(&name, &mount_point, disk_filter, mount_filter) {
                let free_space = disk.available_space();
                let total_space = disk.total_space();
                let used_space = total_space - free_space;

                Some(DiskHarvest {
                    name,
                    mount_point,
                    volume_name,
                    free_space: Some(free_space),
                    used_space: Some(used_space),
                    total_space: Some(total_space),
                })
            } else {
                None
            }
        })
        .collect())
}
