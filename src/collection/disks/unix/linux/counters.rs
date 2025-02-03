//! Based on [heim's implementation](https://github.com/heim-rs/heim/blob/master/heim-disk/src/sys/linux/counters.rs).

use std::{
    fs::File,
    io::{self, BufRead, BufReader},
    num::ParseIntError,
    str::FromStr,
};

use crate::collection::disks::IoCounters;

/// Copied from the `psutil` sources:
///
/// "man iostat" states that sectors are equivalent with blocks and have
/// a size of 512 bytes. Despite this value can be queried at runtime
/// via /sys/block/{DISK}/queue/hw_sector_size and results may vary
/// between 1k, 2k, or 4k... 512 appears to be a magic constant used
/// throughout Linux source code:
/// * <https://stackoverflow.com/a/38136179/376587>
/// * <https://lists.gt.net/linux/kernel/2241060>
/// * <https://github.com/giampaolo/psutil/issues/1305>
/// * <https://github.com/torvalds/linux/blob/4f671fe2f9523a1ea206f63fe60a7c7b3a56d5c7/include/linux/bio.h#L99>
/// * <https://lkml.org/lkml/2015/8/17/234>
const DISK_SECTOR_SIZE: u64 = 512;

impl FromStr for IoCounters {
    type Err = anyhow::Error;

    /// Converts a `&str` to an [`IoCounters`].
    ///
    /// Follows the format used in Linux 2.6+. Note that this completely ignores
    /// the following stats:
    /// - Discard stats from 4.18+
    /// - Flush stats from 5.5+
    ///
    /// <https://www.kernel.org/doc/Documentation/iostats.txt>
    /// <https://www.kernel.org/doc/Documentation/ABI/testing/procfs-diskstats>
    fn from_str(s: &str) -> anyhow::Result<IoCounters> {
        fn next_part<'a>(iter: &mut impl Iterator<Item = &'a str>) -> Result<&'a str, io::Error> {
            iter.next()
                .ok_or_else(|| io::Error::from(io::ErrorKind::InvalidData))
        }

        fn next_part_to_u64<'a>(iter: &mut impl Iterator<Item = &'a str>) -> anyhow::Result<u64> {
            next_part(iter)?
                .parse()
                .map_err(|err: ParseIntError| err.into())
        }

        // Skip the major and minor numbers.
        let mut parts = s.split_whitespace().skip(2);

        let name = next_part(&mut parts)?.to_string();

        // Skip read count, read merged count.
        let mut parts = parts.skip(2);
        let read_bytes = next_part_to_u64(&mut parts)? * DISK_SECTOR_SIZE;

        // Skip read time seconds, write count, and write merged count.
        let mut parts = parts.skip(3);
        let write_bytes = next_part_to_u64(&mut parts)? * DISK_SECTOR_SIZE;

        Ok(IoCounters::new(name, read_bytes, write_bytes))
    }
}

/// Returns an iterator of disk I/O stats. Pulls data from `/proc/diskstats`.
pub fn io_stats() -> anyhow::Result<Vec<IoCounters>> {
    const PROC_DISKSTATS: &str = "/proc/diskstats";

    let mut results = vec![];
    let mut reader = BufReader::new(File::open(PROC_DISKSTATS)?);
    let mut line = String::new();

    // This saves us from doing a string allocation on each iteration compared to
    // `lines()`.
    while let Ok(bytes) = reader.read_line(&mut line) {
        if bytes > 0 {
            if let Ok(counters) = IoCounters::from_str(&line) {
                results.push(counters);
            }
            line.clear();
        } else {
            break;
        }
    }

    #[cfg(feature = "zfs")]
    {
        use crate::collection::disks::zfs_io_counters;
        if let Ok(mut zfs_io) = zfs_io_counters::zfs_io_stats() {
            results.append(&mut zfs_io);
        }
    }

    Ok(results)
}
