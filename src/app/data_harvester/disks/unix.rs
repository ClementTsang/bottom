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

use crate::app::Filter;
use crate::data_harvester::disks::{DiskHarvest, IoData, IoHarvest};

pub fn get_io_usage() -> anyhow::Result<IoHarvest> {
    let mut io_hash: HashMap<String, Option<IoData>> = HashMap::new();

    for io in io_stats()?.flatten() {
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

        let filter_check_map = [(disk_filter, &name), (mount_filter, &mount_point)];

        // This represents case 1. That is, if there is a match in an allowing list - if there is, then
        // immediately allow it!
        let matches_allow_list = filter_check_map.iter().any(|(filter, text)| {
            if let Some(filter) = filter {
                if !filter.is_list_ignored {
                    for r in &filter.list {
                        if r.is_match(text) {
                            return true;
                        }
                    }
                }
            }
            false
        });

        let to_keep = if matches_allow_list {
            true
        } else {
            // If it doesn't match an allow list, then check if it is denied.
            // That is, if it matches in a reject filter, then reject.  Otherwise, we always keep it.
            !filter_check_map.iter().any(|(filter, text)| {
                if let Some(filter) = filter {
                    if filter.is_list_ignored {
                        for r in &filter.list {
                            if r.is_match(text) {
                                return true;
                            }
                        }
                    }
                }
                false
            })
        };

        if to_keep {
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
