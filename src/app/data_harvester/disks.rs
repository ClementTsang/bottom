use crate::app::Filter;

#[derive(Debug, Clone, Default)]
pub struct DiskHarvest {
    pub name: String,
    pub mount_point: String,
    pub free_space: Option<u64>,
    pub used_space: Option<u64>,
    pub total_space: Option<u64>,
}

#[derive(Clone, Debug)]
pub struct IoData {
    pub read_bytes: u64,
    pub write_bytes: u64,
}

pub type IoHarvest = std::collections::HashMap<String, Option<IoData>>;

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

            // FIXME: [MOUNT POINT] Add the filter here I guess?
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
    actually_get: bool, name_filter: &Option<Filter>,
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
            let symlink: std::ffi::OsString;

            let name = (if let Some(device) = partition.device() {
                // See if this disk is actually mounted elsewhere on Linux...
                // This is a workaround to properly map I/O in some cases (i.e. disk encryption), see
                // https://github.com/ClementTsang/bottom/issues/419
                if cfg!(target_os = "linux") {
                    if let Ok(path) = std::fs::read_link(device) {
                        if path.is_absolute() {
                            symlink = path.into_os_string();
                            symlink.as_os_str()
                        } else {
                            let mut combined_path = std::path::PathBuf::new();
                            combined_path.push(device);
                            combined_path.pop(); // Pop the current file...
                            combined_path.push(path.clone());

                            if let Ok(path) = std::fs::canonicalize(combined_path) {
                                // Resolve the local path into an absolute one...
                                symlink = path.into_os_string();
                                symlink.as_os_str()
                            } else {
                                symlink = path.into_os_string();
                                symlink.as_os_str()
                            }
                        }
                    } else {
                        device
                    }
                } else {
                    device
                }
            } else {
                std::ffi::OsStr::new("Name Unavailable")
            }
            .to_str()
            .unwrap_or("Name Unavailable"))
            .to_string();

            let mount_point = (partition
                .mount_point()
                .to_str()
                .unwrap_or("Name Unavailable"))
            .to_string();

            let to_keep = if let Some(filter) = name_filter {
                let mut ret = filter.is_list_ignored;
                for r in &filter.list {
                    if r.is_match(&name) {
                        ret = !filter.is_list_ignored;
                        break;
                    }
                }
                ret
            } else {
                true
            };

            if to_keep {
                // The usage line fails in some cases (Void linux + LUKS, see https://github.com/ClementTsang/bottom/issues/419)
                if let Ok(usage) = heim::disk::usage(partition.mount_point().to_path_buf()).await {
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

    vec_disks.sort_by(|a, b| a.name.cmp(&b.name));

    Ok(Some(vec_disks))
}
