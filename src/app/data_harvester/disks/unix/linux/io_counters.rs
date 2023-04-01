//! Based on [heim's implementation](https://github.com/heim-rs/heim/blob/master/heim-disk/src/sys/linux/counters.rs).

use std::{
    ffi::OsStr,
    fs::File,
    io::{self, BufRead, BufReader},
    str::FromStr,
};

/// Copied from the `psutil` sources:
///
/// "man iostat" states that sectors are equivalent with blocks and have
/// a size of 512 bytes. Despite this value can be queried at runtime
/// via /sys/block/{DISK}/queue/hw_sector_size and results may vary
/// between 1k, 2k, or 4k... 512 appears to be a magic constant used
/// throughout Linux source code:
/// * https://stackoverflow.com/a/38136179/376587
/// * https://lists.gt.net/linux/kernel/2241060
/// * https://github.com/giampaolo/psutil/issues/1305
/// * https://github.com/torvalds/linux/blob/4f671fe2f9523a1ea206f63fe60a7c7b3a56d5c7/include/linux/bio.h#L99
/// * https://lkml.org/lkml/2015/8/17/234
const DISK_SECTOR_SIZE: u64 = 512;

#[allow(dead_code)]
#[derive(Debug, Default)]
pub struct IoStats {
    name: String,
    read_count: u64,
    read_merged_count: u64,
    read_bytes: u64,
    read_time_secs: u64,
    write_count: u64,
    write_merged_count: u64,
    write_bytes: u64,
    write_time_secs: u64,
}

impl IoStats {
    pub(crate) fn device_name(&self) -> &OsStr {
        OsStr::new(&self.name)
    }

    pub(crate) fn read_bytes(&self) -> u64 {
        self.read_bytes
    }

    pub(crate) fn write_bytes(&self) -> u64 {
        self.write_bytes
    }
}

impl FromStr for IoStats {
    type Err = anyhow::Error;

    /// Converts a `&str` to an [`IoStats`].
    ///
    /// Follows the format used in Linux 2.6+. Note that this completely ignores the following stats:
    /// - Discard stats from 4.18+
    /// - Flush stats from 5.5+
    ///
    /// https://www.kernel.org/doc/Documentation/iostats.txt
    /// https://www.kernel.org/doc/Documentation/ABI/testing/procfs-diskstats
    fn from_str(s: &str) -> anyhow::Result<IoStats> {
        fn next_part<'a>(iter: &mut impl Iterator<Item = &'a str>) -> Result<&'a str, io::Error> {
            iter.next()
                .ok_or_else(|| io::Error::from(io::ErrorKind::InvalidData))
        }

        // Skip the major and minor numbers.
        let mut parts = s.split_whitespace().skip(2);

        let name = next_part(&mut parts)?.to_string();

        let read_count = next_part(&mut parts)?.parse()?;
        let read_merged_count = next_part(&mut parts)?.parse()?;
        let read_bytes = next_part(&mut parts)?.parse::<u64>()? * DISK_SECTOR_SIZE;
        let read_time_secs = next_part(&mut parts)?.parse()?;

        let write_count = next_part(&mut parts)?.parse()?;
        let write_merged_count = next_part(&mut parts)?.parse()?;
        let write_bytes = next_part(&mut parts)?.parse::<u64>()? * DISK_SECTOR_SIZE;
        let write_time_secs = next_part(&mut parts)?.parse()?;

        Ok(IoStats {
            name,
            read_count,
            read_merged_count,
            read_bytes,
            read_time_secs,
            write_count,
            write_merged_count,
            write_bytes,
            write_time_secs,
        })
    }
}

/// Returns an iterator of disk I/O stats. Pulls data from `/proc/diskstats`.
pub fn io_stats() -> anyhow::Result<impl Iterator<Item = anyhow::Result<IoStats>>> {
    const PROC_DISKSTATS: &str = "/proc/diskstats";

    Ok(BufReader::new(File::open(PROC_DISKSTATS)?)
        .lines()
        .map(|line| match line {
            Ok(line) => IoStats::from_str(&line),
            Err(err) => Err(err.into()),
        }))
}
