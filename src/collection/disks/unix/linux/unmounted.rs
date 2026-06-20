//! Return block devices that aren't currently mounted on Linux.

use std::{
    collections::HashSet,
    fs::File,
    io::{BufRead, BufReader},
};

use crate::collection::disks::DiskHarvest;

/// `/proc/partitions` reports sizes in 1024-byte blocks.
const PARTITION_BLOCK_SIZE: u64 = 1024;

/// Returns [`DiskHarvest`] entries for block devices that aren't in `mounted`.
///
/// These come from `/proc/partitions`, which lists every block device (so even
/// devices with no I/O activity are covered) along with its size in terms of blocks.
/// Note this also filters out some devices, like `loop*`, `ram*`, `zram*`, etc., as
/// these are not "disks".
pub(crate) fn unmounted_disks(mounted: &HashSet<String>) -> Vec<DiskHarvest> {
    const PROC_PARTITIONS: &str = "/proc/partitions";

    let Ok(file) = File::open(PROC_PARTITIONS) else {
        return Vec::new();
    };

    let mut disks = Vec::new();
    let mut reader = BufReader::new(file);
    let mut line = String::new();

    while let Ok(bytes) = reader.read_line(&mut line) {
        if bytes == 0 {
            break;
        }

        // Format: `major minor #blocks name`. The header line and the blank line
        // after it simply fail to parse here and get skipped.
        let mut parts = line.split_whitespace();
        let blocks = parts.nth(2).and_then(|b| b.parse::<u64>().ok());
        let name = parts.next();

        if let (Some(blocks), Some(name)) = (blocks, name) {
            if !(mounted.contains(name)
                || name.starts_with("loop")
                || name.starts_with("ram")
                || name.starts_with("zram"))
            {
                disks.push(DiskHarvest {
                    name: format!("/dev/{name}"),
                    mount_point: String::new(),
                    free_space: None,
                    used_space: None,
                    total_space: Some(blocks * PARTITION_BLOCK_SIZE),
                });
            }
        }

        line.clear();
    }

    disks
}
