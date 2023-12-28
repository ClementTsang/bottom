//! Memory data collection.

#[cfg(not(target_os = "windows"))]
pub(crate) use self::sysinfo::get_cache_usage;
pub(crate) use self::sysinfo::get_ram_usage;

pub mod sysinfo;
cfg_if::cfg_if! {
    if #[cfg(target_os = "windows")] {
        pub mod windows;
        pub(crate) use self::windows::get_swap_usage;
    } else {
        pub(crate) use self::sysinfo::get_swap_usage;

    }
}

#[cfg(feature = "zfs")]
pub mod arc;

#[derive(Debug, Clone, Default)]
pub struct MemHarvest {
    pub used_bytes: u64,
    pub total_bytes: u64,
    pub use_percent: Option<f64>, // TODO: Might be find to just make this an f64, and any consumer checks NaN.
}
