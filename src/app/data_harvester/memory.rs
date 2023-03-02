//! Data collection for memory.
//!
//! For Linux, macOS, and Windows, this is handled by Heim. On FreeBSD it is handled by sysinfo.

pub mod sysinfo;
pub use self::sysinfo::*;

#[derive(Debug, Clone, Default)]
pub struct MemHarvest {
    pub mem_total_in_kib: u64,
    pub mem_used_in_kib: u64,
    pub use_percent: Option<f64>,
}

#[derive(Debug)]
pub struct MemCollect {
    pub ram: Option<MemHarvest>,
    pub swap: Option<MemHarvest>,
    #[cfg(feature = "zfs")]
    pub arc: Option<MemHarvest>,
    #[cfg(feature = "gpu")]
    pub gpus: Option<Vec<(String, MemHarvest)>>,
}
