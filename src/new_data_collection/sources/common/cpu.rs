//! Common code for retrieving CPU data.

#[derive(Debug, Clone, Copy)]
pub(crate) enum CpuDataType {
    Avg,
    Cpu(usize),
}

/// Represents a single core/thread and its usage.
#[derive(Debug, Clone)]
pub(crate) struct CpuData {
    pub entry_type: CpuDataType,
    pub usage: f64,
}

/// Collected CPU data at an instance.
#[derive(Debug, Clone)]
pub(crate) struct CpuHarvest {
    pub usages: Vec<CpuData>,
    pub load_average: [f32; 3],
}
