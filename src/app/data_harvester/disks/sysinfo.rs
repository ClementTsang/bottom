use serde::Deserialize;
use std::io;

use super::{DiskHarvest, IoHarvest};
use crate::app::Filter;

#[derive(Deserialize, Debug)]
#[serde(rename_all = "kebab-case")]
struct DfXo {
    #[serde(default)]
    storage_system_information: StorageSystemInformation,
}

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

    let io_harvest = get_disk_info().map(|df| {
        df.storage_system_information
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

    let mut vec_disks: Vec<DiskHarvest> = get_disk_info().map(|df: DfXo| {
        df.storage_system_information
            .filesystem
            .into_iter()
            .map(|disk| DiskHarvest {
                free_space: Some(disk.available_blocks * 1024),
                used_space: Some(disk.used_blocks * 1024),
                total_space: Some(disk.total_blocks * 1024),
                mount_point: disk.mounted_on,
                name: disk.name,
            })
            .collect()
    })?;
    //.expect("FIXME");

    // let mut vec_disks: Vec<DiskHarvest> = Vec::new();
    // for disk in sys.disks() {
    //     // Name is expected to be device name
    //     let name = disk.name().to_string_lossy().into();
    //     let mount_point = disk.mount_point().display().to_string();

    //     // TODO implement filters
    //     vec_disks.push(DiskHarvest {
    //         free_space: Some(disk.available_space()),
    //         used_space: Some(disk.total_space().saturating_sub(disk.available_space())),
    //         total_space: Some(disk.total_space()),
    //         mount_point,
    //         name,
    //     });
    // }

    // let partitions_stream = heim::disk::partitions_physical().await?;
    // futures::pin_mut!(partitions_stream);

    // while let Some(part) = partitions_stream.next().await {
    //     if let Ok(partition) = part {
    //         let name = get_device_name(&partition);

    //         let mount_point = (partition
    //             .mount_point()
    //             .to_str()
    //             .unwrap_or("Name Unavailable"))
    //         .to_string();

    //         // Precedence ordering in the case where name and mount filters disagree, "allow" takes precedence over "deny".
    //         //
    //         // For implementation, we do this as follows:
    //         // 1. Is the entry allowed through any filter? That is, does it match an entry in a filter where `is_list_ignored` is `false`? If so, we always keep this entry.
    //         // 2. Is the entry denied through any filter? That is, does it match an entry in a filter where `is_list_ignored` is `true`? If so, we always deny this entry.
    //         // 3. Anything else is allowed.

    //         let filter_check_map = [(disk_filter, &name), (mount_filter, &mount_point)];

    //         // This represents case 1.  That is, if there is a match in an allowing list - if there is, then
    //         // immediately allow it!
    //         let matches_allow_list = filter_check_map.iter().any(|(filter, text)| {
    //             if let Some(filter) = filter {
    //                 if !filter.is_list_ignored {
    //                     for r in &filter.list {
    //                         if r.is_match(text) {
    //                             return true;
    //                         }
    //                     }
    //                 }
    //             }
    //             false
    //         });

    //         let to_keep = if matches_allow_list {
    //             true
    //         } else {
    //             // If it doesn't match an allow list, then check if it is denied.
    //             // That is, if it matches in a reject filter, then reject.  Otherwise, we always keep it.
    //             !filter_check_map.iter().any(|(filter, text)| {
    //                 if let Some(filter) = filter {
    //                     if filter.is_list_ignored {
    //                         for r in &filter.list {
    //                             if r.is_match(text) {
    //                                 return true;
    //                             }
    //                         }
    //                     }
    //                 }
    //                 false
    //             })
    //         };

    //         if to_keep {
    //             // The usage line can fail in some cases (for example, if you use Void Linux + LUKS,
    //             // see https://github.com/ClementTsang/bottom/issues/419 for details).  As such, check
    //             // it like this instead.
    //             if let Ok(usage) = heim::disk::usage(partition.mount_point()).await {
    //                 vec_disks.push(DiskHarvest {
    //                     free_space: Some(usage.free().get::<heim::units::information::byte>()),
    //                     used_space: Some(usage.used().get::<heim::units::information::byte>()),
    //                     total_space: Some(usage.total().get::<heim::units::information::byte>()),
    //                     mount_point,
    //                     name,
    //                 });
    //             } else {
    //                 vec_disks.push(DiskHarvest {
    //                     free_space: None,
    //                     used_space: None,
    //                     total_space: None,
    //                     mount_point,
    //                     name,
    //                 });
    //             }
    //         }
    //     }
    // }

    vec_disks.sort_by(|a, b| a.name.cmp(&b.name));

    Ok(Some(vec_disks))
}

fn get_disk_info() -> io::Result<DfXo> {
    let output = std::process::Command::new("df")
        .args(&["--libxo", "json", "-k"])
        .output()?;
    serde_json::from_slice(&output.stdout).map_err(|err| io::Error::new(io::ErrorKind::Other, err))
}
