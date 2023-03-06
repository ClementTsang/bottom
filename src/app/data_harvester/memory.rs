//! Memory data collection.

pub mod sysinfo;
pub(crate) use self::sysinfo::get_mem_data;

pub mod gpu;
pub(crate) use gpu::get_gpu_data;

pub mod arc;
pub(crate) use arc::get_arc_data;

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
}
