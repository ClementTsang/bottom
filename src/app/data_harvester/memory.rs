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
    pub total_kib: u64,
    pub used_kib: u64,
    pub use_percent: Option<f64>,
}
