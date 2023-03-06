//! Memory data collection.

pub mod sysinfo;
pub(crate) use self::sysinfo::{get_ram_usage, get_swap_usage};

pub mod gpu;
pub(crate) use gpu::get_gpu_mem_usage;

pub mod arc;
pub(crate) use arc::get_arc_usage;

#[derive(Debug, Clone, Default)]
pub struct MemHarvest {
    pub total_kib: u64,
    pub used_kib: u64,
    pub use_percent: Option<f64>,
}
