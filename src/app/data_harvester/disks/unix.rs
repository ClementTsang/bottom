//! Disk stats for Unix-like systems that aren't supported through other means.

mod file_systems;

use file_systems::*;

mod usage;
use usage::*;

cfg_if::cfg_if! {
    if #[cfg(target_os = "linux")] {
        mod linux;
        pub use linux::*;
    } else if #[cfg(target_os = "macos")] {
        mod other;
        use other::*;

        mod macos;
        pub use macos::*;
    } else {
        mod other;
        use other::*;
    }
}

use super::{keep_disk_entry, DiskHarvest};
use crate::app::Filter;

/// Returns the disk usage of the mounted (and for now, physical) disks.
pub fn get_disk_usage(
    disk_filter: &Option<Filter>, mount_filter: &Option<Filter>,
) -> anyhow::Result<Vec<DiskHarvest>> {
    let mut vec_disks: Vec<DiskHarvest> = Vec::new();

    for partition in physical_partitions()? {
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
