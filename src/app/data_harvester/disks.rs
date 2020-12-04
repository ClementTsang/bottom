#[cfg(not(any(target_arch = "aarch64", target_arch = "arm")))]
use futures::stream::StreamExt;

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

#[cfg(any(target_arch = "aarch64", target_arch = "arm"))]
pub async fn get_io_usage(
    _sys: &sysinfo::System,
) -> crate::utils::error::Result<Option<IOHarvest>> {
    let io_hash: std::collections::HashMap<String, Option<IOData>> =
        std::collections::HashMap::new();
    Ok(Some(io_hash))

    // TODO: Sysinfo disk I/O usage.
    // ...sadly, this cannot be done as of now (other than me writing my own), it requires further
    // work.  See https://github.com/GuillaumeGomez/sysinfo/issues/304.
}

#[cfg(any(target_arch = "aarch64", target_arch = "arm"))]
pub async fn get_disk_usage(
    sys: &sysinfo::System,
) -> crate::utils::error::Result<Option<Vec<DiskHarvest>>> {
    use sysinfo::{DiskExt, SystemExt};

    let mut vec_disks = sys
        .get_disks()
        .iter()
        .map(|disk| DiskHarvest {
            name: disk.get_name().to_string_lossy().into(),
            mount_point: disk.get_mount_point().to_string_lossy().into(),
            free_space: disk.get_available_space(),
            used_space: disk
                .get_total_space()
                .saturating_sub(disk.get_available_space()),
            total_space: disk.get_total_space(),
        })
        .collect::<Vec<DiskHarvest>>();
    vec_disks.sort_by(|a, b| a.name.cmp(&b.name));

    Ok(Some(vec_disks))
}

#[cfg(not(any(target_arch = "aarch64", target_arch = "arm")))]
pub async fn get_io_usage(
    get_physical: bool, actually_get: bool,
) -> crate::utils::error::Result<Option<IOHarvest>> {
    if !actually_get {
        return Ok(None);
    }

    let mut io_hash: std::collections::HashMap<String, Option<IOData>> =
        std::collections::HashMap::new();
    if get_physical {
        let mut physical_counter_stream = heim::disk::io_counters_physical();
        while let Some(io) = physical_counter_stream.next().await {
            if let Ok(io) = io {
                let mount_point = io.device_name().to_str().unwrap_or("Name Unavailable");
                io_hash.insert(
                    mount_point.to_string(),
                    Some(IOData {
                        read_bytes: io.read_bytes().get::<heim::units::information::megabyte>(),
                        write_bytes: io.write_bytes().get::<heim::units::information::megabyte>(),
                    }),
                );
            }
        }
    } else {
        let mut counter_stream = heim::disk::io_counters();
        while let Some(io) = counter_stream.next().await {
            if let Ok(io) = io {
                let mount_point = io.device_name().to_str().unwrap_or("Name Unavailable");
                io_hash.insert(
                    mount_point.to_string(),
                    Some(IOData {
                        read_bytes: io.read_bytes().get::<heim::units::information::byte>(),
                        write_bytes: io.write_bytes().get::<heim::units::information::byte>(),
                    }),
                );
            }
        }
    }

    Ok(Some(io_hash))
}

#[cfg(not(any(target_arch = "aarch64", target_arch = "arm")))]
pub async fn get_disk_usage(
    actually_get: bool,
) -> crate::utils::error::Result<Option<Vec<DiskHarvest>>> {
    if !actually_get {
        return Ok(None);
    }

    let mut vec_disks: Vec<DiskHarvest> = Vec::new();
    let mut partitions_stream = heim::disk::partitions_physical();

    while let Some(part) = partitions_stream.next().await {
        if let Ok(part) = part {
            let partition = part;
            let usage = heim::disk::usage(partition.mount_point().to_path_buf()).await?;

            vec_disks.push(DiskHarvest {
                free_space: usage.free().get::<heim::units::information::byte>(),
                used_space: usage.used().get::<heim::units::information::byte>(),
                total_space: usage.total().get::<heim::units::information::byte>(),
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

    Ok(Some(vec_disks))
}
