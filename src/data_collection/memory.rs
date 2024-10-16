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
    pub use_percent: Option<f64>, /* TODO: Might be fine to just make this an f64, and any
                                   * consumer checks NaN. */
}
