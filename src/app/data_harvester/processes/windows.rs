//! Process data collection for Windows.  Uses sysinfo.

use sysinfo::{CpuExt, PidExt, ProcessExt, System, SystemExt, UserExt};

use super::ProcessHarvest;

pub fn get_process_data(
    sys: &System, use_current_cpu_total: bool, unnormalized_cpu: bool, mem_total_kb: u64,
) -> crate::utils::error::Result<Vec<ProcessHarvest>> {
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
        };

        let disk_usage = process_val.disk_usage();
        let process_state = (process_val.status().to_string(), 'R');
        process_vector.push(ProcessHarvest {
            pid: process_val.pid().as_u32() as _,
            parent_pid: process_val.parent().map(|p| p.as_u32() as _),
            name,
            command,
            mem_usage_percent: if mem_total_kb > 0 {
                process_val.memory() as f64 * 100.0 / mem_total_kb as f64
            } else {
                0.0
            },
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
        });
    }

    Ok(process_vector)
}
