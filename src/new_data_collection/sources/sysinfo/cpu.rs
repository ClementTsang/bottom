use std::collections::VecDeque;

use sysinfo::{LoadAvg, System};

use crate::{
    data_collection::cpu::LoadAvgHarvest,
    new_data_collection::sources::cpu::{CpuData, CpuDataType},
};

pub(crate) fn get_cpu_data_list(sys: &System, show_average_cpu: bool) -> Vec<CpuData> {
    let mut cpu_deque: VecDeque<_> = sys
        .cpus()
        .iter()
        .enumerate()
        .map(|(i, cpu)| CpuData {
            entry_type: CpuDataType::Cpu(i),
            usage: cpu.cpu_usage() as f64,
        })
        .collect();

    if show_average_cpu {
        let cpu = sys.global_cpu_info();

        cpu_deque.push_front(CpuData {
            entry_type: CpuDataType::Avg,
            usage: cpu.cpu_usage() as f64,
        })
    }

    Vec::from(cpu_deque)
}

#[cfg(not(target_os = "windows"))]
pub(crate) fn get_load_avg() -> LoadAvgHarvest {
    // The API for sysinfo apparently wants you to call it like this, rather than
    // using a &System.
    let LoadAvg { one, five, fifteen } = sysinfo::System::load_average();

    [one as f32, five as f32, fifteen as f32]
}
