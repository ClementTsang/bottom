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

    parse_unmounted_disks(BufReader::new(file), mounted)
}

/// Parses `/proc/partitions` into [`DiskHarvest`] entries for block devices not present in `mounted`.
fn parse_unmounted_disks<R: BufRead>(mut reader: R, mounted: &HashSet<String>) -> Vec<DiskHarvest> {
    let mut disks = Vec::new();
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

        if let (Some(blocks), Some(name)) = (blocks, name)
            && !(mounted.contains(name)
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

        line.clear();
    }

    disks
}

#[cfg(test)]
mod tests {
    use super::*;

    // Note the header + blank line at the top, exactly like the real file.
    const SAMPLE: &str = "\
major minor  #blocks  name

   8        0        500 sda
   8        1       1000 sda1
   8        2    4000000 sda2
 259        0   90000000 nvme0n1
 259        1     900000 nvme0n1p1
   7        0     100000 loop0
   1        0       6000 ram0
 252        0    1000000 zram0
   8       16          0 sdb
";

    fn names(disks: &[DiskHarvest]) -> Vec<&str> {
        disks.iter().map(|d| d.name.as_str()).collect()
    }

    #[test]
    fn skip_non_disk_entries() {
        let disks = parse_unmounted_disks(SAMPLE.as_bytes(), &HashSet::new());
        assert_eq!(
            names(&disks),
            vec![
                "/dev/sda",
                "/dev/sda1",
                "/dev/sda2",
                "/dev/nvme0n1",
                "/dev/nvme0n1p1",
                "/dev/sdb",
            ]
        );
    }

    #[test]
    fn excludes_mounted_devices() {
        let mounted = HashSet::from(["sda2".to_string(), "nvme0n1p1".to_string()]);
        let disks = parse_unmounted_disks(SAMPLE.as_bytes(), &mounted);

        assert!(!names(&disks).contains(&"/dev/sda2"));
        assert!(!names(&disks).contains(&"/dev/nvme0n1p1"));

        // Whole disk + other partitions still appear (kept on purpose).
        assert!(names(&disks).contains(&"/dev/sda"));
        assert!(names(&disks).contains(&"/dev/nvme0n1"));
    }

    #[test]
    fn check_disk_harvest_entry() {
        let disks = parse_unmounted_disks(SAMPLE.as_bytes(), &HashSet::new());
        let sda = disks.iter().find(|d| d.name == "/dev/sda").unwrap();

        assert_eq!(sda.total_space, Some(500 * PARTITION_BLOCK_SIZE));
        assert_eq!(sda.mount_point, "");
        assert_eq!(sda.free_space, None);
        assert_eq!(sda.used_space, None);
    }

    #[test]
    fn zero_block_entry() {
        let disks = parse_unmounted_disks(SAMPLE.as_bytes(), &HashSet::new());
        let sdb = disks.iter().find(|d| d.name == "/dev/sdb").unwrap();
        assert_eq!(sdb.total_space, Some(0));
    }
}
