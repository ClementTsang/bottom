//! Memory data collection.

use std::num::NonZeroU64;

pub(crate) use self::sysinfo::get_ram_usage;

pub mod sysinfo;

cfg_select! {
    target_os = "windows" => {
        mod windows;
        pub(crate) use self::windows::get_swap_usage;
    }
    _ => {
        pub(crate) use self::sysinfo::{get_cache_usage, get_swap_usage};
    }
}

#[cfg(feature = "zfs")]
pub mod arc;

#[cfg(target_os = "linux")]
pub mod zram;

#[cfg(target_os = "linux")]
pub mod zswap;

/// Memory usage data.
#[derive(Debug, Clone)]
pub struct MemData {
    pub used_bytes: u64,
    pub total_bytes: NonZeroU64,
}

/// Memory usage data for things that store compressed data
/// (e.g. zram, zswap).
#[cfg(target_os = "linux")]
#[derive(Debug, Clone)]
pub struct CompressedMemData {
    /// The usage to show as the gauge and graph value.
    pub usage: MemData,
    pub compressed_bytes: u64,
    pub original_bytes: u64,
}

impl MemData {
    /// Return the use percentage.
    #[inline]
    pub fn percentage(&self) -> f64 {
        let used = self.used_bytes as f64;
        let total = self.total_bytes.get() as f64;

        used / total * 100.0
    }
}
