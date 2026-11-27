//! Basic [zswap](https://docs.kernel.org/admin-guide/mm/zswap.html) data collection on Linux.

use std::{fs, num::NonZeroU64};

use super::{CompressedMemData, MemData};

/// zswap-related data.
#[derive(Debug, Clone, Default)]
pub struct ZswapHarvest {
    /// The maximum size of the compressed pool, from `max_pool_percent`. In
    /// bytes.
    pub(crate) pool_limit_bytes: u64,
    /// Uncompressed size of the stored pages, parsed from `Zswapped`. In bytes.
    pub(crate) original_size_bytes: u64,
    /// Compressed size of the stored pages, parsed from `Zswap`. In bytes.
    pub(crate) compressed_size_bytes: u64,
}

impl ZswapHarvest {
    /// Return zswap usage as a [`MemData`]. Used bytes are in terms of
    /// the compressed size, while total bytes are the pool limit.
    pub fn as_mem_data(&self) -> Option<MemData> {
        NonZeroU64::new(self.pool_limit_bytes).map(|total_bytes| MemData {
            used_bytes: self.compressed_size_bytes,
            total_bytes,
        })
    }

    /// Return zswap usage along with the compressed and uncompressed sizes.
    pub fn compressed_mem_data(&self) -> Option<CompressedMemData> {
        Some(CompressedMemData {
            usage: self.as_mem_data()?,
            compressed_bytes: self.compressed_size_bytes,
            original_bytes: self.original_size_bytes,
        })
    }
}

/// Zswap usage. Everything is in bytes.
#[derive(Debug, PartialEq, Eq)]
struct ZswapUsedBytes {
    compressed_bytes: u64,
    original_bytes: u64,
}

/// Read zswap usage information from a `meminfo`-formatted string, if it
/// is available.
///
/// Note that these only exist on kernel 5.19+ with zswap support.
fn parse_meminfo(contents: &str) -> Option<ZswapUsedBytes> {
    let mut compressed_bytes = None;
    let mut original_bytes = None;

    for line in contents.lines() {
        let Some((key, value)) = line.split_once(':') else {
            continue;
        };

        let field = match key {
            "Zswap" => &mut compressed_bytes,
            "Zswapped" => &mut original_bytes,
            _ => continue,
        };

        let kib_value: u64 = value
            .trim()
            .trim_end_matches("kB")
            .trim_end()
            .parse()
            .ok()?;
        *field = Some(kib_value * 1024);
    }

    Some(ZswapUsedBytes {
        compressed_bytes: compressed_bytes?,
        original_bytes: original_bytes?,
    })
}

/// Return the pool limit given `max_pool_percent`, falling back to the total
/// memory if it is missing or invalid.
fn pool_limit(max_pool_percent: Option<&str>, mem_total_bytes: u64) -> u64 {
    match max_pool_percent.and_then(|p| p.trim().parse::<u64>().ok()) {
        Some(percent) if percent <= 100 => mem_total_bytes * percent / 100,
        _ => mem_total_bytes,
    }
}

/// Returns zswap usage.
///
/// Returns `None` if zswap is unsupported, or if it is disabled and we can't
/// get any details.
pub(crate) fn get_zswap_usage(mem_total_bytes: u64) -> Option<ZswapHarvest> {
    let zswap_usage = parse_meminfo(&fs::read_to_string("/proc/meminfo").ok()?)?;

    // If this can't be read, assume it's enabled since the meminfo fields exist.
    let zswap_enabled = fs::read_to_string("/sys/module/zswap/parameters/enabled")
        .map(|s| !s.trim_start().starts_with(['N', 'n', '0']))
        .unwrap_or(true);

    // Note that if zswap gets disabled at runtime, it won't evict still-existing
    // data, so we'll continue showing until that is cleared.
    if !zswap_enabled && zswap_usage.compressed_bytes == 0 && zswap_usage.original_bytes == 0 {
        return None;
    }

    let max_pool_percent = fs::read_to_string("/sys/module/zswap/parameters/max_pool_percent").ok();

    Some(ZswapHarvest {
        pool_limit_bytes: pool_limit(max_pool_percent.as_deref(), mem_total_bytes),
        original_size_bytes: zswap_usage.original_bytes,
        compressed_size_bytes: zswap_usage.compressed_bytes,
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parsing_meminfo() {
        let s = "MemTotal:       16303928 kB\nMemFree:         1234567 kB\nZswap:             12345 kB\nZswapped:          54321 kB\n";
        assert_eq!(
            parse_meminfo(s),
            Some(ZswapUsedBytes {
                compressed_bytes: 12345 * 1024,
                original_bytes: 54321 * 1024,
            })
        );

        assert_eq!(
            parse_meminfo("MemTotal:       16303928 kB\nZswap:             12345 kB\n"),
            None
        );
    }
}
