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
pub struct IoCounters {
    name: String,
    read_bytes: u64,
    write_bytes: u64,
}

impl IoCounters {
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

impl FromStr for IoCounters {
    type Err = anyhow::Error;

    /// Converts a `&str` to an [`IoStats`].
    ///
    /// Follows the format used in Linux 2.6+. Note that this completely ignores the following stats:
    /// - Discard stats from 4.18+
    /// - Flush stats from 5.5+
    ///
    /// https://www.kernel.org/doc/Documentation/iostats.txt
    /// https://www.kernel.org/doc/Documentation/ABI/testing/procfs-diskstats
    fn from_str(s: &str) -> anyhow::Result<IoCounters> {
        fn next_part<'a>(iter: &mut impl Iterator<Item = &'a str>) -> Result<&'a str, io::Error> {
            iter.next()
                .ok_or_else(|| io::Error::from(io::ErrorKind::InvalidData))
        }

        // Skip the major and minor numbers.
        let mut parts = s.split_whitespace().skip(2);

        let name = next_part(&mut parts)?.to_string();

        let _read_count = next_part(&mut parts)?.parse()?;
        let _read_merged_count = next_part(&mut parts)?.parse()?;
        let read_bytes = next_part(&mut parts)?.parse::<u64>()? * DISK_SECTOR_SIZE;
        let _read_time_secs = next_part(&mut parts)?.parse()?;

        let _write_count = next_part(&mut parts)?.parse()?;
        let _write_merged_count = next_part(&mut parts)?.parse()?;
        let write_bytes = next_part(&mut parts)?.parse::<u64>()? * DISK_SECTOR_SIZE;
        let _write_time_secs = next_part(&mut parts)?.parse()?;

        Ok(IoCounters {
            name,
            read_bytes,
            write_bytes,
        })
    }
}

/// Returns an iterator of disk I/O stats. Pulls data from `/proc/diskstats`.
pub fn io_stats() -> anyhow::Result<Vec<anyhow::Result<IoCounters>>> {
    const PROC_DISKSTATS: &str = "/proc/diskstats";

    let mut results = vec![];
    let mut reader = BufReader::new(File::open(PROC_DISKSTATS)?);
    let mut line = String::new();

    while let Ok(bytes) = reader.read_line(&mut line) {
        if bytes > 0 {
            results.push(IoCounters::from_str(&line));
            line.clear();
        } else {
            break;
        }
    }

    Ok(results)
}
