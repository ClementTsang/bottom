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
    pub usage: f32,
}

#[derive(Debug, Clone, Default)]
pub struct CpuHarvest {
    pub inner: Vec<CpuData>,
    pub brand: String,
}

impl std::ops::Deref for CpuHarvest {
    type Target = Vec<CpuData>;

    fn deref(&self) -> &Self::Target {
        &self.inner
    }
}

impl std::ops::DerefMut for CpuHarvest {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.inner
    }
}
