//! CPU stats through sysinfo.
//! Supports FreeBSD.

use sysinfo::System;

use super::CpuHarvest;
use crate::collection::error::CollectionResult;

pub fn get_cpu_data_list(sys: &System, show_average_cpu: bool) -> CollectionResult<CpuHarvest> {
    let avg = show_average_cpu.then(|| sys.global_cpu_usage());

    let cpus = sys
        .cpus()
        .iter()
        .map(|cpu| cpu.cpu_usage())
        .collect::<Vec<_>>();

    Ok(CpuHarvest { avg, cpus })
}

#[cfg(target_family = "unix")]
pub(crate) fn get_load_avg() -> crate::collection::cpu::LoadAvgHarvest {
    // The API for sysinfo apparently wants you to call it like this, rather than
    // using a &System.
    let sysinfo::LoadAvg { one, five, fifteen } = sysinfo::System::load_average();

    [one as f32, five as f32, fifteen as f32]
}
