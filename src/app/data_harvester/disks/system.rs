//! Disk stats for Unix-like systems that aren't supported through other means.

mod file_systems;
use std::collections::HashMap;

use file_systems::*;

mod usage;
use usage::*;

cfg_if::cfg_if! {
    if #[cfg(target_os = "linux")] {
        mod linux;
        use linux::*;
    } else {
        mod other;
        use other::*;
    }
}

use super::{keep_disk_entry, DiskHarvest, IoData, IoHarvest};
use crate::app::Filter;

/// Returns the I/O usage of certain mount points.
pub fn get_io_usage() -> anyhow::Result<IoHarvest> {
    let mut io_hash: HashMap<String, Option<IoData>> = HashMap::new();

    for io in io_stats()?.into_iter().flatten() {
        let mount_point = io.device_name().to_string_lossy();

        io_hash.insert(
            mount_point.to_string(),
            Some(IoData {
                read_bytes: io.read_bytes(),
                write_bytes: io.write_bytes(),
            }),
        );
    }

    Ok(io_hash)
}

/// Returns the disk usage of the mounted (and for now, physical) disks.
pub fn get_disk_usage(
    disk_filter: &Option<Filter>, mount_filter: &Option<Filter>,
) -> anyhow::Result<Vec<DiskHarvest>> {
    let mut vec_disks: Vec<DiskHarvest> = Vec::new();

    for partition in partitions_physical()? {
        let name = partition.get_device_name();
        let mount_point = partition.mount_point().to_string_lossy().to_string();

        // Precedence ordering in the case where name and mount filters disagree, "allow" takes precedence over "deny".
        //
        // For implementation, we do this as follows:
        // 1. Is the entry allowed through any filter? That is, does it match an entry in a filter where `is_list_ignored` is `false`? If so, we always keep this entry.
        // 2. Is the entry denied through any filter? That is, does it match an entry in a filter where `is_list_ignored` is `true`? If so, we always deny this entry.
        // 3. Anything else is allowed.

        if keep_disk_entry(&name, &mount_point, disk_filter, mount_filter) {
            // The usage line can fail in some cases (for example, if you use Void Linux + LUKS,
            // see https://github.com/ClementTsang/bottom/issues/419 for details).
            if let Ok(usage) = partition.usage() {
                let total = usage.total();

                vec_disks.push(DiskHarvest {
                    free_space: Some(usage.free()),
                    used_space: Some(total - usage.available()),
                    total_space: Some(total),
                    mount_point,
                    name,
                });
            } else {
                vec_disks.push(DiskHarvest {
                    free_space: None,
                    used_space: None,
                    total_space: None,
                    mount_point,
                    name,
                });
            }
        }
    }

    Ok(vec_disks)
}
