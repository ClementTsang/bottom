//! Data collection for CPU usage and load average.

pub mod sysinfo;
pub use self::sysinfo::*;

pub type LoadAvgHarvest = [f32; 3];

#[derive(Debug, Clone, Copy)]
pub enum CpuDataType {
    Avg,
    Cpu(u32),
}

#[derive(Debug, Clone)]
pub struct CpuData {
    pub data_type: CpuDataType,
    pub usage: f32,
}

#[derive(Debug, Clone, Default)]
pub struct CpuHarvest {
    pub avg: Option<f32>,
    pub cpus: Vec<f32>,
}
