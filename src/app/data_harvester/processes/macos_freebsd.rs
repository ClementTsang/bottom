//! Shared process data harvesting code from macOS and FreeBSD via sysinfo.

use std::collections::HashMap;
use std::io;

use sysinfo::{CpuExt, PidExt, ProcessExt, ProcessStatus, System, SystemExt};

use super::ProcessHarvest;
use crate::{data_harvester::processes::UserTable, utils::error::Result, Pid};

pub fn get_process_data<F>(
    sys: &System, use_current_cpu_total: bool, unnormalized_cpu: bool, total_memory: u64,
    user_table: &mut UserTable, backup_cpu_proc_usage: F,
) -> Result<Vec<ProcessHarvest>>
where
    F: Fn(&[Pid]) -> io::Result<HashMap<Pid, f64>>,
{
    let mut process_vector: Vec<ProcessHarvest> = Vec::new();
    let process_hashmap = sys.processes();
    let cpu_usage = sys.global_cpu_info().cpu_usage() as f64 / 100.0;
    let num_processors = sys.cpus().len() as f64;

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
            if unnormalized_cpu || num_processors == 0.0 {
                usage
            } else {
                usage / num_processors
            }
        };
        let process_cpu_usage = if use_current_cpu_total && cpu_usage > 0.0 {
            pcu / cpu_usage
        } else {
            pcu
        };

        let disk_usage = process_val.disk_usage();
        let process_state = {
            let ps = process_val.status();
            (ps.to_string(), convert_process_status_to_char(ps))
        };
        let uid = process_val.user_id().map(|u| **u);
        let pid = process_val.pid().as_u32() as Pid;
        process_vector.push(ProcessHarvest {
            pid,
            parent_pid: {
                #[cfg(target_os = "macos")]
                {
                    process_val
                        .parent()
                        .map(|p| p.as_u32() as _)
                        .or_else(|| super::fallback_macos_ppid(pid))
                }
                #[cfg(not(target_os = "macos"))]
                {
                    process_val.parent().map(|p| p.as_u32() as _)
                }
            },
            name,
            command,
            mem_usage_percent: if total_memory > 0 {
                process_val.memory() as f64 * 100.0 / total_memory as f64
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
            uid,
            user: uid
                .and_then(|uid| {
                    user_table
                        .get_uid_to_username_mapping(uid)
                        .map(Into::into)
                        .ok()
                })
                .unwrap_or_else(|| "N/A".into()),
        });
    }

    let unknown_state = ProcessStatus::Unknown(0).to_string();
    let cpu_usage_unknown_pids: Vec<Pid> = process_vector
        .iter()
        .filter(|process| process.process_state.0 == unknown_state)
        .map(|process| process.pid)
        .collect();
    let cpu_usages = backup_cpu_proc_usage(&cpu_usage_unknown_pids)?;
    for process in &mut process_vector {
        if cpu_usages.contains_key(&process.pid) {
            process.cpu_usage_percent = if unnormalized_cpu || num_processors == 0.0 {
                *cpu_usages.get(&process.pid).unwrap()
            } else {
                *cpu_usages.get(&process.pid).unwrap() / num_processors
            };
        }
    }

    Ok(process_vector)
}

fn convert_process_status_to_char(status: ProcessStatus) -> char {
    match status {
        ProcessStatus::Run => 'R',
        ProcessStatus::Sleep => 'S',
        ProcessStatus::Idle => 'D',
        ProcessStatus::Zombie => 'Z',
        _ => '?',
    }
}
