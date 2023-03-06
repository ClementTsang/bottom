//! Memory data collection.

pub mod sysinfo;
pub(crate) use self::sysinfo::{get_ram_usage, get_swap_usage};

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
