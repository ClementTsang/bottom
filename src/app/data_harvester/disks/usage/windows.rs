//! Disk stats via sysinfo.

use sysinfo::{DiskExt, System, SystemExt};

use crate::app::filter::Filter;
use crate::data_harvester::disks::DiskHarvest;

pub(crate) fn get_disk_usage(
    sys: &System, disk_filter: &Option<Filter>, mount_filter: &Option<Filter>,
) -> Vec<DiskHarvest> {
    let disks = sys.disks();
    disks
        .iter()
        .filter_map(|disk| {
            let name = disk
                .name()
                .to_os_string()
                .into_string()
                .unwrap_or_else(|_| "Name Unavailable".to_string());

            let mount_point = disk
                .mount_point()
                .as_os_str()
                .to_os_string()
                .into_string()
                .unwrap_or_else(|_| "Name Unavailable".to_string());

            if keep_entry(&name, &mount_point, disk_filter, mount_filter) {
                let free_space = disk.available_space();
                let total_space = disk.total_space();
                let used_space = total_space - free_space;

                Some(DiskHarvest {
                    name,
                    mount_point,
                    free_space,
                    used_space,
                    total_space,
                })
            } else {
                None
            }
        })
        .collect()
}

fn keep_entry(
    name: &str, mount_point: &str, disk_filter: &Option<Filter>, mount_filter: &Option<Filter>,
) -> bool {
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

    to_keep
}
