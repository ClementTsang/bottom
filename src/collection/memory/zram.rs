//! Basic zram data collection on Linux.

use std::{fs, ops::AddAssign, path::Path};

/// zram-related data.
///
/// **Note:** at the moment, this is the sum of _all_ zram devices.
#[derive(Debug, Clone, Default)]
pub struct ZramHarvest {
    /// Total uncompressed capacity, from `disksize`. In bytes.
    pub capacity_bytes: u64,
    /// Uncompressed size of the stored data. In bytes.
    pub orig_data_bytes: u64,
    /// Compressed size of the stored data. In bytes.
    pub compressed_data_bytes: u64,
    /// Actual RAM used, including allocator overhead. In bytes.
    pub mem_used_bytes: u64,
}

impl AddAssign for ZramHarvest {
    fn add_assign(&mut self, rhs: Self) {
        self.capacity_bytes += rhs.capacity_bytes;
        self.orig_data_bytes += rhs.orig_data_bytes;
        self.compressed_data_bytes += rhs.compressed_data_bytes;
        self.mem_used_bytes += rhs.mem_used_bytes;
    }
}

/// Read the first three fields of `mm_stat` and return them as bytes:
///
/// ```none
/// orig_data_size compr_data_size mem_used_total ...
/// ```
fn parse_mm_stat(contents: &str) -> Option<(u64, u64, u64)> {
    let mut fields = contents
        .split_ascii_whitespace()
        .map(|f| f.parse::<u64>().ok());

    Some((fields.next()??, fields.next()??, fields.next()??))
}

/// Reads a given path representing a zram device, returning a partial [`ZramHarvest`].
fn read_device(dev: &Path) -> Option<ZramHarvest> {
    let disk_size: u64 = fs::read_to_string(dev.join("disksize"))
        .ok()?
        .trim()
        .parse()
        .ok()?;

    // An uninitialized device has a disksize of 0.
    if disk_size == 0 {
        return None;
    }

    let (orig_data_bytes, compressed_data_bytes, mem_used_bytes) =
        parse_mm_stat(&fs::read_to_string(dev.join("mm_stat")).ok()?)?;

    Some(ZramHarvest {
        capacity_bytes: disk_size,
        orig_data_bytes,
        compressed_data_bytes,
        mem_used_bytes,
    })
}

/// Returns aggregated zram usage. Returns `None` if there are no initialized zram
/// devices.
pub(crate) fn get_zram_usage() -> Option<ZramHarvest> {
    let mut result = ZramHarvest::default();
    let mut found = false;

    for entry in fs::read_dir("/sys/block").ok()?.flatten() {
        if !entry.file_name().as_encoded_bytes().starts_with(b"zram") {
            continue;
        }

        if let Some(partial_result) = read_device(&entry.path()) {
            result += partial_result;
            found = true;
        }
    }

    found.then_some(result)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parses_mm_stat() {
        let s = "  4567890   1345678   1234567        0   1234567      321        0        0        0\n";
        assert_eq!(parse_mm_stat(s), Some((4567890, 1345678, 1234567)));
    }
}
