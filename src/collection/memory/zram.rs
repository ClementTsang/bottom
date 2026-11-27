//! Basic [zram](https://docs.kernel.org/admin-guide/blockdev/zram.html) data collection on Linux.

use std::{fs, num::NonZeroU64, ops::AddAssign, path::Path};

use super::MemData;

/// zram-related data.
///
/// **Note:** at the moment, this is the sum of _all_ zram devices.
#[derive(Debug, Clone, Default)]
pub struct ZramHarvest {
    /// Total uncompressed capacity, from `disksize`. In bytes.
    pub(crate) capacity_bytes: u64,
    /// Uncompressed size of the stored data, including same-filled pages. In
    /// bytes.
    pub(crate) original_size_bytes: u64,
    /// Compressed size of the stored data. In bytes.
    pub(crate) compressed_size_bytes: u64,
    /// Actual RAM used, including allocator overhead. In bytes.
    pub(crate) mem_used_bytes: u64,
}

impl ZramHarvest {
    /// Return zram usage in terms of used and total bytes.
    pub fn as_mem_data(&self) -> Option<MemData> {
        NonZeroU64::new(self.capacity_bytes).map(|total_bytes| MemData {
            used_bytes: self.original_size_bytes,
            total_bytes,
        })
    }

    /// Return the compressed size of the stored data, capped to the
    /// uncompressed size. In bytes.
    pub fn compressed_bytes(&self) -> u64 {
        self.compressed_size_bytes.min(self.original_size_bytes)
    }
}

impl AddAssign for ZramHarvest {
    fn add_assign(&mut self, rhs: Self) {
        self.capacity_bytes += rhs.capacity_bytes;
        self.original_size_bytes += rhs.original_size_bytes;
        self.compressed_size_bytes += rhs.compressed_size_bytes;
        self.mem_used_bytes += rhs.mem_used_bytes;
    }
}

/// The fields we get from `mm_stat`.
#[derive(Debug, PartialEq, Eq)]
struct MmStat {
    orig_data_bytes: u64,
    compressed_data_bytes: u64,
    mem_used_bytes: u64,
}

/// Read the first three fields of `mm_stat`, which are all in bytes:
///
/// ```none
/// orig_data_size compr_data_size mem_used_total ...
/// ```
fn parse_mm_stat(contents: &str) -> Option<MmStat> {
    let mut fields = contents
        .split_ascii_whitespace()
        .map(|f| f.parse::<u64>().ok());

    let orig_data_bytes = fields.next()??;
    let compressed_data_bytes = fields.next()??;
    let mem_used_bytes = fields.next()??;

    Some(MmStat {
        orig_data_bytes,
        compressed_data_bytes,
        mem_used_bytes,
    })
}

/// Reads a given path representing a zram device, returning a partial [`ZramHarvest`].
fn read_device(dev: &Path) -> Option<ZramHarvest> {
    let disk_size: u64 = fs::read_to_string(dev.join("disksize"))
        .ok()?
        .trim()
        .parse()
        .ok()?;

    // An uninitialized device has a `disksize` of 0.
    if disk_size == 0 {
        return None;
    }

    let mm_stat = parse_mm_stat(&fs::read_to_string(dev.join("mm_stat")).ok()?)?;

    Some(ZramHarvest {
        capacity_bytes: disk_size,
        original_size_bytes: mm_stat.orig_data_bytes,
        compressed_size_bytes: mm_stat.compressed_data_bytes,
        mem_used_bytes: mm_stat.mem_used_bytes,
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
        assert_eq!(
            parse_mm_stat(s),
            Some(MmStat {
                orig_data_bytes: 4567890,
                compressed_data_bytes: 1345678,
                mem_used_bytes: 1234567,
            })
        );
    }

    #[test]
    fn compressed_bytes_is_capped() {
        let zram = ZramHarvest {
            original_size_bytes: 10,
            compressed_size_bytes: 20,
            ..Default::default()
        };
        assert_eq!(zram.compressed_bytes(), 10);
    }
}
