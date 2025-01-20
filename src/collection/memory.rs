//! Memory data collection.

#[cfg(not(target_os = "windows"))]
pub(crate) use self::sysinfo::get_cache_usage;
pub(crate) use self::sysinfo::{get_ram_usage, get_swap_usage};

pub mod sysinfo;
// cfg_if::cfg_if! {
//     if #[cfg(target_os = "windows")] {
//         mod windows;
//         pub(crate) use self::windows::get_committed_usage;
//     }
// }

#[cfg(feature = "zfs")]
pub mod arc;

#[derive(Debug, Clone, Default)]
pub struct MemHarvest {
    pub used_bytes: u64,
    pub total_bytes: u64,
}

impl MemHarvest {
    /// Return the use percentage. If the total bytes is 0, then this returns `None`.
    #[inline]
    pub fn checked_percent(&self) -> Option<f64> {
        let used = self.used_bytes as f64;
        let total = self.total_bytes as f64;

        if total == 0.0 {
            None
        } else {
            Some(used / total * 100.0)
        }
    }

    /// Return the use percentage. If the total bytes is 0, then this returns 0.0.
    #[inline]
    pub fn saturating_percentage(&self) -> f64 {
        let used = self.used_bytes as f64;
        let total = self.total_bytes as f64;

        if total == 0.0 {
            0.0
        } else {
            used / total * 100.0
        }
    }
}
