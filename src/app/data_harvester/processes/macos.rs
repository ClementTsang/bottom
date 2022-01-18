//! Process data collection for macOS.  Uses sysinfo.

use super::ProcessHarvest;
use sysinfo::{PidExt, ProcessExt, ProcessStatus, ProcessorExt, System, SystemExt};

fn get_macos_process_cpu_usage(
    pids: &[i32],
) -> std::io::Result<std::collections::HashMap<i32, f64>> {
    use itertools::Itertools;
    let output = std::process::Command::new("ps")
        .args(&["-o", "pid=,pcpu=", "-p"])
        .arg(
            // Has to look like this since otherwise, it you hit a `unstable_name_collisions` warning.
            Itertools::intersperse(pids.iter().map(i32::to_string), ",".to_string())
                .collect::<String>(),
        )
        .output()?;
    let mut result = std::collections::HashMap::new();
    String::from_utf8_lossy(&output.stdout)
        .split_whitespace()
        .chunks(2)
        .into_iter()
        .for_each(|chunk| {
            let chunk: Vec<&str> = chunk.collect();
            if chunk.len() != 2 {
                panic!("Unexpected `ps` output");
            }
            let pid = chunk[0].parse();
            let usage = chunk[1].parse();
            if let (Ok(pid), Ok(usage)) = (pid, usage) {
                result.insert(pid, usage);
            }
        });
    Ok(result)
}

pub fn get_process_data(
    sys: &System, use_current_cpu_total: bool, mem_total_kb: u64,
) -> crate::utils::error::Result<Vec<ProcessHarvest>> {
    let mut process_vector: Vec<ProcessHarvest> = Vec::new();
    let process_hashmap = sys.processes();
    let cpu_usage = sys.global_processor_info().cpu_usage() as f64 / 100.0;
    let num_processors = sys.processors().len() as f64;
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
            let p = process_val.cpu_usage() as f64 / num_processors;
            if p.is_nan() {
                process_val.cpu_usage() as f64
            } else {
                p
            }
        };
        let process_cpu_usage = if use_current_cpu_total && cpu_usage > 0.0 {
            pcu / cpu_usage
        } else {
            pcu
        };

        let disk_usage = process_val.disk_usage();
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
            mem_usage_bytes: process_val.memory() * 1024,
            cpu_usage_percent: process_cpu_usage,
            read_bytes_per_sec: disk_usage.read_bytes,
            write_bytes_per_sec: disk_usage.written_bytes,
            total_read_bytes: disk_usage.total_read_bytes,
            total_write_bytes: disk_usage.total_written_bytes,
            process_state: process_val.status().to_string(),
            process_state_char: convert_process_status_to_char(process_val.status()),
            uid: Some(process_val.uid),
        });
    }

    let unknown_state = ProcessStatus::Unknown(0).to_string();
    let cpu_usage_unknown_pids: Vec<i32> = process_vector
        .iter()
        .filter(|process| process.process_state == unknown_state)
        .map(|process| process.pid)
        .collect();
    let cpu_usages = get_macos_process_cpu_usage(&cpu_usage_unknown_pids)?;
    for process in &mut process_vector {
        if cpu_usages.contains_key(&process.pid) {
            process.cpu_usage_percent = if num_processors == 0.0 {
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
