//! Disk stats through heim.
//! Supports macOS, Linux, and Windows.

use crate::app::Filter;
use crate::data_harvester::disks::{DiskHarvest, IoData, IoHarvest};

cfg_if::cfg_if! {
    if #[cfg(target_os = "linux")] {
        pub mod linux;
        pub use linux::*;
    } else if #[cfg(any(target_os = "macos", target_os = "windows"))] {
        pub mod windows_macos;
        pub use windows_macos::*;
    }
}

pub async fn get_io_usage(actually_get: bool) -> crate::utils::error::Result<Option<IoHarvest>> {
    if !actually_get {
        return Ok(None);
    }

    use futures::StreamExt;

    let mut io_hash: std::collections::HashMap<String, Option<IoData>> =
        std::collections::HashMap::new();

    let counter_stream = heim::disk::io_counters().await?;
    futures::pin_mut!(counter_stream);

    while let Some(io) = counter_stream.next().await {
        if let Ok(io) = io {
            let mount_point = io.device_name().to_str().unwrap_or("Name Unavailable");

            io_hash.insert(
                mount_point.to_string(),
                Some(IoData {
                    read_bytes: io.read_bytes().get::<heim::units::information::byte>(),
                    write_bytes: io.write_bytes().get::<heim::units::information::byte>(),
                }),
            );
        }
    }

    Ok(Some(io_hash))
}

pub async fn get_disk_usage(
    actually_get: bool, disk_filter: &Option<Filter>, mount_filter: &Option<Filter>,
) -> crate::utils::error::Result<Option<Vec<DiskHarvest>>> {
    if !actually_get {
        return Ok(None);
    }

    use futures::StreamExt;

    let mut vec_disks: Vec<DiskHarvest> = Vec::new();
    let partitions_stream = heim::disk::partitions_physical().await?;
    futures::pin_mut!(partitions_stream);

    while let Some(part) = partitions_stream.next().await {
        if let Ok(partition) = part {
            let name = get_device_name(&partition);

            let mount_point = (partition
                .mount_point()
                .to_str()
                .unwrap_or("Name Unavailable"))
            .to_string();

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
                // see https://github.com/ClementTsang/bottom/issues/419 for details).  As such, check
                // it like this instead.
                if let Ok(usage) = heim::disk::usage(partition.mount_point()).await {
                    vec_disks.push(DiskHarvest {
                        free_space: Some(usage.free().get::<heim::units::information::byte>()),
                        used_space: Some(usage.used().get::<heim::units::information::byte>()),
                        total_space: Some(usage.total().get::<heim::units::information::byte>()),
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
    }

    Ok(Some(vec_disks))
}
