//! Disk stats for FreeBSD.

use std::io;

use hashbrown::HashMap;
use serde::Deserialize;

use super::{DiskHarvest, IoHarvest, keep_disk_entry};
use crate::collection::{DataCollector, deserialize_xo, disks::IoData, error::CollectionResult};

#[derive(Deserialize, Debug, Default)]
#[serde(rename_all = "kebab-case")]
struct StorageSystemInformation {
    filesystem: Vec<FileSystem>,
}

#[derive(Deserialize, Debug)]
#[serde(rename_all = "kebab-case")]
struct FileSystem {
    name: String,
    total_blocks: u64,
    used_blocks: u64,
    available_blocks: u64,
    mounted_on: String,
}

pub fn get_io_usage() -> CollectionResult<IoHarvest> {
    // TODO: Should this (and other I/O collectors) fail fast? In general, should
    // collection ever fail fast?
    let mut io_harvest: HashMap<String, Option<IoData>> =
        get_disk_info().map(|storage_system_information| {
            storage_system_information
                .filesystem
                .into_iter()
                .map(|disk| (disk.name, None))
                .collect()
        })?;

    #[cfg(feature = "zfs")]
    {
        use crate::collection::disks::zfs_io_counters;
        if let Ok(zfs_io) = zfs_io_counters::zfs_io_stats() {
            for io in zfs_io.into_iter() {
                let mount_point = io.device_name().to_string_lossy();
                io_harvest.insert(
                    mount_point.to_string(),
                    Some(IoData {
                        read_bytes: io.read_bytes(),
                        write_bytes: io.write_bytes(),
                    }),
                );
            }
        }
    }
    Ok(io_harvest)
}

pub fn get_disk_usage(collector: &DataCollector) -> CollectionResult<Vec<DiskHarvest>> {
    let disk_filter = &collector.filters.disk_filter;
    let mount_filter = &collector.filters.mount_filter;
    let vec_disks: Vec<DiskHarvest> = get_disk_info().map(|storage_system_information| {
        storage_system_information
            .filesystem
            .into_iter()
            .filter_map(|disk| {
                if keep_disk_entry(&disk.name, &disk.mounted_on, disk_filter, mount_filter) {
                    Some(DiskHarvest {
                        free_space: Some(disk.available_blocks * 1024),
                        used_space: Some(disk.used_blocks * 1024),
                        total_space: Some(disk.total_blocks * 1024),
                        mount_point: disk.mounted_on,
                        name: disk.name,
                    })
                } else {
                    None
                }
            })
            .collect()
    })?;

    Ok(vec_disks)
}

fn get_disk_info() -> io::Result<StorageSystemInformation> {
    // TODO: Ideally we don't have to shell out to a new program.
    let output = std::process::Command::new("df")
        .args(["--libxo", "json", "-k", "-t", "ufs,msdosfs,zfs"])
        .output()?;
    deserialize_xo("storage-system-information", &output.stdout)
}
