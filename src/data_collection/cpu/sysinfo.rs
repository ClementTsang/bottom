//! CPU stats through sysinfo.
//! Supports FreeBSD.

use std::collections::VecDeque;

use sysinfo::{LoadAvg, System};

use super::{CpuData, CpuDataType, CpuHarvest};
use crate::data_collection::cpu::LoadAvgHarvest;

pub fn get_cpu_data_list(sys: &System, show_average_cpu: bool) -> crate::error::Result<CpuHarvest> {
    let mut cpu_deque: VecDeque<_> = sys
        .cpus()
        .iter()
        .enumerate()
        .map(|(i, cpu)| CpuData {
            data_type: CpuDataType::Cpu(i),
            cpu_usage: cpu.cpu_usage() as f64,
        })
        .collect();

    if show_average_cpu {
        let cpu = sys.global_cpu_info();

        cpu_deque.push_front(CpuData {
            data_type: CpuDataType::Avg,
            cpu_usage: cpu.cpu_usage() as f64,
        })
    }

    Ok(Vec::from(cpu_deque))
}

pub fn get_load_avg() -> crate::error::Result<LoadAvgHarvest> {
    // The API for sysinfo apparently wants you to call it like this, rather than
    // using a &System.
    let LoadAvg { one, five, fifteen } = sysinfo::System::load_average();

    Ok([one as f32, five as f32, fifteen as f32])
}
