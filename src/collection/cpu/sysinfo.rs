//! CPU stats through sysinfo.

use super::{CpuData, CpuDataType, CpuHarvest};
use crate::collection::{DataCollector, error::CollectionResult};

pub fn get_cpu_data_list(collector: &DataCollector) -> CollectionResult<CpuHarvest> {
    let sys = &collector.sys.system;
    let show_average_cpu = collector.show_average_cpu;

    let mut cpus = vec![];

    if show_average_cpu {
        cfg_select! {
            target_os = "linux" => {
                cpus.push(CpuData {
                    data_type: CpuDataType::Avg,
                    usage: collector
                        .cgroup_cpu_data
                        .avg_cpu_percent
                        .unwrap_or_else(|| sys.global_cpu_usage()),
                });
            }
            _ => cpus.push(CpuData {
                data_type: CpuDataType::Avg,
                usage: sys.global_cpu_usage(),
            }),
        }
    }

    cpus.extend(
        sys.cpus()
            .iter()
            .enumerate()
            .map(|(i, cpu)| CpuData {
                data_type: CpuDataType::Cpu(i),
                usage: cpu.usage(),
            })
            .collect::<Vec<_>>(),
    );

    Ok(cpus)
}

#[cfg(unix)]
pub(crate) fn get_load_avg() -> crate::collection::cpu::LoadAvgHarvest {
    // The API for sysinfo apparently wants you to call it like this, rather
    // than using a &System.
    let (one, five, fifteen) = match sysinfo::System::load_average() {
        Ok(avg) => (avg.one as f32, avg.five as f32, avg.fifteen as f32),
        Err(_) => (0.0, 0.0, 0.0),
    };

    [one, five, fifteen]
}
