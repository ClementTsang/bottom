//! Disk stats via sysinfo.

use sysinfo::{DiskExt, System, SystemExt};

use super::{keep_disk_entry, DiskHarvest, IoHarvest};
use crate::{app::filter::Filter, utils::error};

pub fn get_io_usage() -> error::Result<IoHarvest> {
    Ok(IoHarvest::default())
}

pub(crate) fn get_disk_usage(
    sys: &System, disk_filter: &Option<Filter>, mount_filter: &Option<Filter>,
) -> Vec<DiskHarvest> {
    let disks = sys.disks();
    disks
        .iter()
        .filter_map(|disk| {
            let name = {
                let name = disk.name();

                if name.is_empty() {
                    "Name unavailable".to_string()
                } else {
                    name.to_os_string()
                        .into_string()
                        .unwrap_or_else(|_| "Name unavailable".to_string())
                }
            };

            let mount_point = disk
                .mount_point()
                .as_os_str()
                .to_os_string()
                .into_string()
                .unwrap_or_else(|_| "Mount unavailable".to_string());

            if keep_disk_entry(&name, &mount_point, disk_filter, mount_filter) {
                let free_space = Some(disk.available_space());
                let total_space = Some(disk.total_space());
                let used_space = Some(total_space - free_space);

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
