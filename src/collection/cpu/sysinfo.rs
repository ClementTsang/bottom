//! CPU stats through sysinfo.

use super::{CpuData, CpuDataType, CpuHarvest};
use crate::collection::{DataCollector, error::CollectionResult};

pub fn get_cpu_data_list(collector: &DataCollector) -> CollectionResult<CpuHarvest> {
    let sys = &collector.sys.system;
    let show_average_cpu = collector.show_average_cpu;

    let mut cpus = vec![];

    if show_average_cpu {
        cfg_if::cfg_if! {
            if #[cfg(target_os = "linux")] {
                cpus.push(CpuData {
                    data_type: CpuDataType::Avg,
                    usage: collector.cgroup_cpu_data.avg_cpu_percent.unwrap_or_else(|| sys.global_cpu_usage()),
                });
            } else {
                cpus.push(CpuData {
                    data_type: CpuDataType::Avg,
                    usage: sys.global_cpu_usage(),
                })
            }
        }
    }

    cpus.extend(
        sys.cpus()
            .iter()
            .enumerate()
            .map(|(i, cpu)| CpuData {
                data_type: CpuDataType::Cpu(i),
                usage: cpu.cpu_usage(),
            })
            .collect::<Vec<_>>(),
    );

    Ok(cpus)
}

#[cfg(unix)]
pub(crate) fn get_load_avg() -> crate::collection::cpu::LoadAvgHarvest {
    // The API for sysinfo apparently wants you to call it like this, rather than
    // using a &System.
    let sysinfo::LoadAvg { one, five, fifteen } = sysinfo::System::load_average();

    [one as f32, five as f32, fifteen as f32]
}
