//! Process data collection for Windows. Uses sysinfo.

use std::time::Duration;

use sysinfo::{CpuExt, PidExt, ProcessExt, SystemExt, UserExt};

use super::ProcessHarvest;

use crate::data_harvester::DataCollector;

pub fn sysinfo_process_data(
    collector: &mut DataCollector,
) -> crate::utils::error::Result<Vec<ProcessHarvest>> {
    let sys = &collector.sys;
    let use_current_cpu_total = collector.use_current_cpu_total;
    let unnormalized_cpu = collector.unnormalized_cpu;
    let total_memory = collector.total_memory();

    let mut process_vector: Vec<ProcessHarvest> = Vec::new();
    let process_hashmap = sys.processes();
    let cpu_usage = sys.global_cpu_info().cpu_usage() as f64 / 100.0;
    let num_processors = sys.cpus().len();

    for process_val in process_hashmap.values() {
        let name = if process_val.name().is_empty() {
            let process_cmd = process_val.cmd();
            if process_cmd.len() > 1 {
                process_cmd[0].clone()
            } else {
                let process_exe = process_val.exe().file_stem();
                if let Some(exe) = process_exe {
                    let process_exe_opt = exe.to_str();
                    if let Some(exe_name) = process_exe_opt {
                        exe_name.to_string()
                    } else {
                        "".to_string()
                    }
                } else {
                    "".to_string()
                }
            }
        } else {
            process_val.name().to_string()
        };
        let command = {
            let command = process_val.cmd().join(" ");
            if command.is_empty() {
                name.to_string()
            } else {
                command
            }
        };

        let pcu = {
            let usage = process_val.cpu_usage() as f64;
            if unnormalized_cpu || num_processors == 0 {
                usage
            } else {
                usage / (num_processors as f64)
            }
        };
        let process_cpu_usage = if use_current_cpu_total && cpu_usage > 0.0 {
            pcu / cpu_usage
        } else {
            pcu
        } as f32;

        let disk_usage = process_val.disk_usage();
        let process_state = (process_val.status().to_string(), 'R');

        #[cfg(feature = "gpu")]
        let (gpu_mem, gpu_util, gpu_mem_percent) = {
            let mut gpu_mem = 0;
            let mut gpu_util = 0;
            let mut gpu_mem_percent = 0.0;
            if let Some(gpus) = &collector.gpu_pids {
                gpus.iter().for_each(|gpu| {
                    // add mem/util for all gpus to pid
                    if let Some((mem, util)) = gpu.get(&process_val.pid().as_u32()) {
                        gpu_mem += mem;
                        gpu_util += util;
                    }
                });
            }
            if let Some(gpu_total_mem) = &collector.gpus_total_mem {
                gpu_mem_percent = (gpu_mem as f64 / *gpu_total_mem as f64 * 100.0) as f32;
            }
            (gpu_mem, gpu_util, gpu_mem_percent)
        };
        process_vector.push(ProcessHarvest {
            pid: process_val.pid().as_u32() as _,
            parent_pid: process_val.parent().map(|p| p.as_u32() as _),
            name,
            command,
            mem_usage_percent: if total_memory > 0 {
                process_val.memory() as f64 * 100.0 / total_memory as f64
            } else {
                0.0
            } as f32,
            mem_usage_bytes: process_val.memory(),
            cpu_usage_percent: process_cpu_usage,
            read_bytes_per_sec: disk_usage.read_bytes,
            write_bytes_per_sec: disk_usage.written_bytes,
            total_read_bytes: disk_usage.total_read_bytes,
            total_write_bytes: disk_usage.total_written_bytes,
            process_state,
            user: process_val
                .user_id()
                .and_then(|uid| sys.get_user_by_id(uid))
                .map_or_else(|| "N/A".into(), |user| user.name().to_owned().into()),
            time: if process_val.start_time() == 0 {
                // Workaround for Windows occasionally returning a start time equal to UNIX epoch, giving a run time
                // in the range of 50+ years. We just return a time of zero in this case for simplicity.
                Duration::ZERO
            } else {
                Duration::from_secs(process_val.run_time())
            },
            #[cfg(feature = "gpu")]
            gpu_mem,
            #[cfg(feature = "gpu")]
            gpu_util,
            #[cfg(feature = "gpu")]
            gpu_mem_percent,
        });
    }

    Ok(process_vector)
}
