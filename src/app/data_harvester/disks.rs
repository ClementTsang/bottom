use futures::stream::StreamExt;
use heim::units::information;

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

pub type IOHarvest = std::collections::HashMap<String, IOData>;

pub async fn get_io_usage_list(get_physical: bool) -> crate::utils::error::Result<IOHarvest> {
    let mut io_hash: std::collections::HashMap<String, IOData> = std::collections::HashMap::new();
    if get_physical {
        let mut physical_counter_stream = heim::disk::io_counters_physical();
        while let Some(io) = physical_counter_stream.next().await {
            let io = io?;
            let mount_point = io.device_name().to_str().unwrap_or("Name Unavailable");
            io_hash.insert(
                mount_point.to_string(),
                IOData {
                    read_bytes: io.read_bytes().get::<information::megabyte>(),
                    write_bytes: io.write_bytes().get::<information::megabyte>(),
                },
            );
        }
    } else {
        let mut counter_stream = heim::disk::io_counters();
        while let Some(io) = counter_stream.next().await {
            let io = io?;
            let mount_point = io.device_name().to_str().unwrap_or("Name Unavailable");
            io_hash.insert(
                mount_point.to_string(),
                IOData {
                    read_bytes: io.read_bytes().get::<information::byte>(),
                    write_bytes: io.write_bytes().get::<information::byte>(),
                },
            );
        }
    }

    Ok(io_hash)
}

pub async fn get_disk_usage_list() -> crate::utils::error::Result<Vec<DiskHarvest>> {
    let mut vec_disks: Vec<DiskHarvest> = Vec::new();
    let mut partitions_stream = heim::disk::partitions_physical();

    while let Some(part) = partitions_stream.next().await {
        if let Ok(part) = part {
            let partition = part;
            let usage = heim::disk::usage(partition.mount_point().to_path_buf()).await?;

            vec_disks.push(DiskHarvest {
                free_space: usage.free().get::<information::byte>(),
                used_space: usage.used().get::<information::byte>(),
                total_space: usage.total().get::<information::byte>(),
                mount_point: (partition
                    .mount_point()
                    .to_str()
                    .unwrap_or("Name Unavailable"))
                .to_string(),
                name: (partition
                    .device()
                    .unwrap_or_else(|| std::ffi::OsStr::new("Name Unavailable"))
                    .to_str()
                    .unwrap_or("Name Unavailable"))
                .to_string(),
            });
        }
    }

    vec_disks.sort_by(|a, b| a.name.cmp(&b.name));

    Ok(vec_disks)
}
