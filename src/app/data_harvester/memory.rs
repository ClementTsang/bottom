//! Memory data collection.

pub mod sysinfo;
pub(crate) use self::sysinfo::get_ram_usage;

cfg_if::cfg_if! {
    if #[cfg(target_os = "windows")] {
        pub mod windows;
        pub(crate) use self::windows::get_swap_usage;
    } else {
        pub(crate) use self::sysinfo::get_swap_usage;

    }
}

#[cfg(feature = "gpu")]
pub mod gpu;

#[cfg(feature = "zfs")]
pub mod arc;

#[derive(Debug, Clone, Default)]
pub struct MemHarvest {
    pub used_bytes: u64,
    pub total_bytes: u64,
    pub use_percent: Option<f64>, // TODO: Might be find to just make this an f64, and any consumer checks NaN.
}
