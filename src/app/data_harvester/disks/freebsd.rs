//! Disk stats for FreeBSD.

use std::io;

use serde::Deserialize;

use super::{keep_disk_entry, DiskHarvest, IoData, IoHarvest};
use crate::{app::Filter, data_harvester::deserialize_xo, utils::error};

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

pub fn get_io_usage() -> error::Result<IoHarvest> {
    let io_harvest = get_disk_info().map(|storage_system_information| {
        storage_system_information
            .filesystem
            .into_iter()
            .map(|disk| (disk.name, None))
            .collect()
    })?;

    Ok(io_harvest)
}

pub fn get_disk_usage(
    disk_filter: &Option<Filter>, mount_filter: &Option<Filter>,
) -> error::Result<Vec<DiskHarvest>> {
    let vec_disks: Vec<DiskHarvest> = get_disk_info().map(|storage_system_information| {
        storage_system_information
            .filesystem
            .into_iter()
            .filter_map(|disk| {
                if keep_disk_entry(&disk.name, &disk.mounted_on, disk_filter, mount_filter) {
                    Some(DiskHarvest {
                        free_space: disk.available_blocks * 1024,
                        used_space: disk.used_blocks * 1024,
                        total_space: disk.total_blocks * 1024,
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
