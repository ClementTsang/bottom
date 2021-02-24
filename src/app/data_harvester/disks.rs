use crate::app::Filter;

#[derive(Debug, Clone, Default)]
pub struct DiskHarvest {
    pub name: String,
    pub mount_point: String,
    pub free_space: u64,
    pub used_space: u64,
    pub total_space: u64,
}

#[derive(Clone, Debug)]
pub struct IOData {
    pub read_bytes: u64,
    pub write_bytes: u64,
}

pub type IOHarvest = std::collections::HashMap<String, Option<IOData>>;

pub async fn get_io_usage(actually_get: bool) -> crate::utils::error::Result<Option<IOHarvest>> {
    if !actually_get {
        return Ok(None);
    }

    use futures::StreamExt;

    let mut io_hash: std::collections::HashMap<String, Option<IOData>> =
        std::collections::HashMap::new();

    let counter_stream = heim::disk::io_counters().await?;
    futures::pin_mut!(counter_stream);

    while let Some(io) = counter_stream.next().await {
        if let Ok(io) = io {
            let mount_point = io.device_name().to_str().unwrap_or("Name Unavailable");

            // FIXME: [MOUNT POINT] Add the filter here I guess?

            io_hash.insert(
                mount_point.to_string(),
                Some(IOData {
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
            let name = (partition
                .device()
                .unwrap_or_else(|| std::ffi::OsStr::new("Name Unavailable"))
                .to_str()
                .unwrap_or("Name Unavailable"))
            .to_string();
            debug!("Partition name: {}", name);

            let mount_point = (partition
                .mount_point()
                .to_str()
                .unwrap_or("Name Unavailable"))
            .to_string();
            debug!("Mount point: {}", name);

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
                let usage = heim::disk::usage(partition.mount_point().to_path_buf()).await?;
                vec_disks.push(DiskHarvest {
                    free_space: usage.free().get::<heim::units::information::byte>(),
                    used_space: usage.used().get::<heim::units::information::byte>(),
                    total_space: usage.total().get::<heim::units::information::byte>(),
                    mount_point,
                    name,
                });
            }
        }
    }

    vec_disks.sort_by(|a, b| a.name.cmp(&b.name));

    Ok(Some(vec_disks))
}
