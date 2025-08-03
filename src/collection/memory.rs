//! Memory data collection.

use std::num::NonZeroU64;

pub(crate) use self::sysinfo::get_ram_usage;

pub mod sysinfo;

cfg_if::cfg_if! {
    if #[cfg(target_os = "windows")] {
        mod windows;
        pub(crate) use self::windows::get_swap_usage;
    } else {
        pub(crate) use self::sysinfo::{get_cache_usage, get_swap_usage};
    }
}

#[cfg(feature = "zfs")]
pub mod arc;

#[derive(Debug, Clone)]
pub struct MemData {
    pub used_bytes: u64,
    pub total_bytes: NonZeroU64,
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
