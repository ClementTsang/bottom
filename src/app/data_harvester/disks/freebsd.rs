//! Disk stats for FreeBSD.

use std::io;

use serde::Deserialize;

use super::{DiskHarvest, IoHarvest};
use crate::app::Filter;
use crate::data_harvester::deserialize_xo;

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

pub async fn get_io_usage(actually_get: bool) -> crate::utils::error::Result<Option<IoHarvest>> {
    if !actually_get {
        return Ok(None);
    }

    let io_harvest = get_disk_info().map(|storage_system_information| {
        storage_system_information
            .filesystem
            .into_iter()
            .map(|disk| (disk.name, None))
            .collect()
    })?;
    Ok(Some(io_harvest))
}

pub async fn get_disk_usage(
    actually_get: bool, disk_filter: &Option<Filter>, mount_filter: &Option<Filter>,
) -> crate::utils::error::Result<Option<Vec<DiskHarvest>>> {
    if !actually_get {
        return Ok(None);
    }

    let vec_disks: Vec<DiskHarvest> = get_disk_info().map(|storage_system_information| {
        storage_system_information
            .filesystem
            .into_iter()
            .filter_map(|disk| {
                // Precedence ordering in the case where name and mount filters disagree, "allow"
                // takes precedence over "deny".
                //
                // For implementation, we do this as follows:
                //
                // 1. Is the entry allowed through any filter? That is, does it match an entry in a
                //    filter where `is_list_ignored` is `false`? If so, we always keep this entry.
                // 2. Is the entry denied through any filter? That is, does it match an entry in a
                //    filter where `is_list_ignored` is `true`? If so, we always deny this entry.
                // 3. Anything else is allowed.
                let filter_check_map =
                    [(disk_filter, &disk.name), (mount_filter, &disk.mounted_on)];
                if matches_allow_list(filter_check_map.as_slice())
                    || !matches_ignore_list(filter_check_map.as_slice())
                {
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

    Ok(Some(vec_disks))
}

fn matches_allow_list(filter_check_map: &[(&Option<Filter>, &String)]) -> bool {
    filter_check_map.iter().any(|(filter, text)| match filter {
        Some(f) if !f.is_list_ignored => f.list.iter().any(|r| r.is_match(text)),
        Some(_) | None => false,
    })
}

fn matches_ignore_list(filter_check_map: &[(&Option<Filter>, &String)]) -> bool {
    filter_check_map.iter().any(|(filter, text)| match filter {
        Some(f) if f.is_list_ignored => f.list.iter().any(|r| r.is_match(text)),
        Some(_) | None => false,
    })
}

fn get_disk_info() -> io::Result<StorageSystemInformation> {
    // TODO: Ideally we don't have to shell out to a new program.
    let output = std::process::Command::new("df")
        .args(["--libxo", "json", "-k", "-t", "ufs,msdosfs,zfs"])
        .output()?;
    deserialize_xo("storage-system-information", &output.stdout)
}
