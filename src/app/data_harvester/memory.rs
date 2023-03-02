//! Memory data collection.

pub mod sysinfo;
pub(crate) use self::sysinfo::*;

#[derive(Debug, Clone, Default)]
pub struct MemHarvest {
    pub total_kib: u64,
    pub used_kib: u64,
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
