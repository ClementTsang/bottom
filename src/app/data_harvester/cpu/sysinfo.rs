//! CPU stats through sysinfo.
//! Supports FreeBSD.

use std::collections::VecDeque;

use sysinfo::{LoadAvg, ProcessorExt, System, SystemExt};

use super::{CpuData, CpuHarvest, PastCpuTotal, PastCpuWork};
use crate::app::data_harvester::cpu::LoadAvgHarvest;

pub async fn get_cpu_data_list(
    sys: &sysinfo::System, show_average_cpu: bool,
    _previous_cpu_times: &mut Vec<(PastCpuWork, PastCpuTotal)>,
    _previous_average_cpu_time: &mut Option<(PastCpuWork, PastCpuTotal)>,
) -> crate::error::Result<CpuHarvest> {
    let mut cpu_deque: VecDeque<_> = sys
        .processors()
        .iter()
        .enumerate()
        .map(|(i, cpu)| CpuData {
            cpu_prefix: "CPU".to_string(),
            cpu_count: Some(i),
            cpu_usage: cpu.cpu_usage() as f64,
        })
        .collect();

    if show_average_cpu {
        let cpu = sys.global_processor_info();

        cpu_deque.push_front(CpuData {
            cpu_prefix: "AVG".to_string(),
            cpu_count: None,
            cpu_usage: cpu.cpu_usage() as f64,
        })
    }

    Ok(Vec::from(cpu_deque))
}

pub async fn get_load_avg() -> crate::error::Result<LoadAvgHarvest> {
    let sys = System::new();
    let LoadAvg { one, five, fifteen } = sys.load_average();

    Ok([one as f32, five as f32, fifteen as f32])
}
