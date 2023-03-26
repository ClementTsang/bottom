//! Implementation based on [heim's](https://github.com/heim-rs/heim)
//! Unix disk usage.

mod partition;
pub(crate) use partition::*;

use std::{
    fs::File,
    io::{BufRead, BufReader},
    str::FromStr,
};

fn get_device_name(partition: &Partition) -> String {
    if let Some(device) = partition.device() {
        // See if this disk is actually mounted elsewhere on Linux. This is a workaround
        // to properly map I/O in some cases (i.e. disk encryption), see https://github.com/ClementTsang/bottom/issues/419
        if let Ok(path) = std::fs::read_link(device) {
            if path.is_absolute() {
                path.into_os_string()
                    .into_string()
                    .unwrap_or_else(|_| "Name unavailable".to_string())
            } else {
                let mut combined_path = std::path::PathBuf::new();
                combined_path.push(device);
                combined_path.pop(); // Pop the current file...
                combined_path.push(path);

                if let Ok(canon_path) = std::fs::canonicalize(combined_path) {
                    // Resolve the local path into an absolute one...
                    canon_path
                        .into_os_string()
                        .into_string()
                        .unwrap_or_else(|_| "Name unavailable".to_string())
                } else {
                    device.to_owned()
                }
            }
        } else {
            device.to_owned()
        }
    } else {
        "Name unavailable".to_string()
    }
}

/// Returns all partitions.
pub(crate) fn partitions() -> anyhow::Result<Vec<Partition>> {
    let mounts = BufReader::new(File::open("/proc/mounts")?).lines();
    Ok(mounts
        .filter_map(|line| match line {
            Ok(line) => Partition::from_str(&line).ok(),
            Err(_) => None,
        })
        .collect())
}

/// Returns all physical partitions.
pub(crate) fn partitions_physical() -> anyhow::Result<Vec<Partition>> {
    let mounts = BufReader::new(File::open("/proc/mounts")?).lines();
    Ok(mounts
        .filter_map(|line| match line {
            Ok(line) => Partition::from_str(&line).ok(),
            Err(_) => None,
        })
        .filter(|partition| partition.fs_type().is_physical())
        .collect())
}
