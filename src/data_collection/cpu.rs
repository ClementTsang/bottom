//! Data collection for CPU usage and load average.

pub mod sysinfo;
pub use self::sysinfo::*;

pub type LoadAvgHarvest = [f32; 3];

#[derive(Debug, Clone, Copy)]
pub enum CpuDataType {
    Avg,
    Cpu(usize),
}

#[derive(Debug, Clone)]
pub struct CpuData {
    pub data_type: CpuDataType,
    pub cpu_usage: f64,
}

pub type CpuHarvest = Vec<CpuData>;
