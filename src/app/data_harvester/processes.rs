use std::{
    collections::{hash_map::RandomState, HashMap},
    process::Command,
    time::Instant,
};

use sysinfo::{ProcessExt, ProcessorExt, System, SystemExt};

use crate::utils::error;

#[derive(Clone)]
pub enum ProcessSorting {
    CPU,
    MEM,
    PID,
    NAME,
}

impl Default for ProcessSorting {
    fn default() -> Self {
        ProcessSorting::CPU
    }
}

#[derive(Debug, Clone, Default)]
pub struct ProcessHarvest {
    pub pid: u32,
    pub cpu_usage_percent: f64,
    pub mem_usage_percent: f64,
    pub name: String,
}

fn cpu_usage_calculation(
    prev_idle: &mut f64, prev_non_idle: &mut f64,
) -> error::Result<(f64, f64)> {
    // From SO answer: https://stackoverflow.com/a/23376195
    let mut path = std::path::PathBuf::new();
    path.push("/proc");
    path.push("stat");

    let stat_results = std::fs::read_to_string(path)?;
    let first_line: &str;

    let split_results = stat_results.split('\n').collect::<Vec<&str>>();
    if split_results.is_empty() {
        return Err(error::BottomError::InvalidIO(format!(
            "Unable to properly split the stat results; saw {} values, expected at least 1 value.",
            split_results.len()
        )));
    } else {
        first_line = split_results[0];
    }

    let val = first_line.split_whitespace().collect::<Vec<&str>>();

    // SC in case that the parsing will fail due to length:
    if val.len() <= 10 {
        return Err(error::BottomError::InvalidIO(format!(
            "CPU parsing will fail due to too short of a return value; saw {} values, expected 10 values.",
            val.len()
        )));
    }

    let user: f64 = val[1].parse::<_>().unwrap_or(0_f64);
    let nice: f64 = val[2].parse::<_>().unwrap_or(0_f64);
    let system: f64 = val[3].parse::<_>().unwrap_or(0_f64);
    let idle: f64 = val[4].parse::<_>().unwrap_or(0_f64);
    let iowait: f64 = val[5].parse::<_>().unwrap_or(0_f64);
    let irq: f64 = val[6].parse::<_>().unwrap_or(0_f64);
    let softirq: f64 = val[7].parse::<_>().unwrap_or(0_f64);
    let steal: f64 = val[8].parse::<_>().unwrap_or(0_f64);
    let guest: f64 = val[9].parse::<_>().unwrap_or(0_f64);

    let idle = idle + iowait;
    let non_idle = user + nice + system + irq + softirq + steal + guest;

    let total = idle + non_idle;
    let prev_total = *prev_idle + *prev_non_idle;

    let total_delta: f64 = total - prev_total;
    let idle_delta: f64 = idle - *prev_idle;

    //debug!("Vangelis function: CPU PERCENT: {}", (total_delta - idle_delta) / total_delta * 100_f64);

    *prev_idle = idle;
    *prev_non_idle = non_idle;

    let result = if total_delta - idle_delta != 0_f64 {
        total_delta - idle_delta
    } else {
        1_f64
    };

    let cpu_percentage = if total_delta != 0_f64 {
        result / total_delta
    } else {
        0_f64
    };

    Ok((result, cpu_percentage))
}

fn get_process_cpu_stats(pid: u32) -> std::io::Result<f64> {
    let mut path = std::path::PathBuf::new();
    path.push("/proc");
    path.push(&pid.to_string());
    path.push("stat");

    let stat_results = std::fs::read_to_string(path)?;
    let val = stat_results.split_whitespace().collect::<Vec<&str>>();
    let utime = val[13].parse::<f64>().unwrap_or(0_f64);
    let stime = val[14].parse::<f64>().unwrap_or(0_f64);

    //debug!("PID: {}, utime: {}, stime: {}", pid, utime, stime);

    Ok(utime + stime) // This seems to match top...
}

/// Note that cpu_fraction should be represented WITHOUT the \times 100 factor!
fn linux_cpu_usage<S: core::hash::BuildHasher>(
    pid: u32, cpu_usage: f64, cpu_fraction: f64,
    prev_pid_stats: &HashMap<String, (f64, Instant), S>,
    new_pid_stats: &mut HashMap<String, (f64, Instant), S>, use_current_cpu_total: bool,
    curr_time: Instant,
) -> std::io::Result<f64> {
    // Based heavily on https://stackoverflow.com/a/23376195 and https://stackoverflow.com/a/1424556
    let before_proc_val: f64 = if prev_pid_stats.contains_key(&pid.to_string()) {
        prev_pid_stats
            .get(&pid.to_string())
            .unwrap_or(&(0_f64, curr_time))
            .0
    } else {
        0_f64
    };
    let after_proc_val = get_process_cpu_stats(pid)?;

    /*debug!(
        "PID - {} - Before: {}, After: {}, CPU: {}, Percentage: {}",
        pid,
        before_proc_val,
        after_proc_val,
        cpu_usage,
        (after_proc_val - before_proc_val) / cpu_usage * 100_f64
    );*/

    new_pid_stats.insert(pid.to_string(), (after_proc_val, curr_time));

    if use_current_cpu_total {
        Ok((after_proc_val - before_proc_val) / cpu_usage * 100_f64)
    } else {
        Ok((after_proc_val - before_proc_val) / cpu_usage * 100_f64 * cpu_fraction)
    }
}

fn convert_ps<S: core::hash::BuildHasher>(
    process: &str, cpu_usage: f64, cpu_fraction: f64,
    prev_pid_stats: &HashMap<String, (f64, Instant), S>,
    new_pid_stats: &mut HashMap<String, (f64, Instant), S>, use_current_cpu_total: bool,
    curr_time: Instant,
) -> std::io::Result<ProcessHarvest> {
    if process.trim().to_string().is_empty() {
        return Ok(ProcessHarvest {
            pid: 0,
            name: "".to_string(),
            mem_usage_percent: 0.0,
            cpu_usage_percent: 0.0,
        });
    }

    let pid = (&process[..11])
        .trim()
        .to_string()
        .parse::<u32>()
        .unwrap_or(0);
    let name = (&process[11..61]).trim().to_string();
    let mem_usage_percent = (&process[62..])
        .trim()
        .to_string()
        .parse::<f64>()
        .unwrap_or(0_f64);

    let cpu_usage_percent = linux_cpu_usage(
        pid,
        cpu_usage,
        cpu_fraction,
        prev_pid_stats,
        new_pid_stats,
        use_current_cpu_total,
        curr_time,
    )?;
    Ok(ProcessHarvest {
        pid,
        name,
        mem_usage_percent,
        cpu_usage_percent,
    })
}

pub fn get_sorted_processes_list(
    sys: &System, prev_idle: &mut f64, prev_non_idle: &mut f64,
    prev_pid_stats: &mut HashMap<String, (f64, Instant), RandomState>, use_current_cpu_total: bool,
    mem_total_kb: u64, curr_time: Instant,
) -> crate::utils::error::Result<Vec<ProcessHarvest>> {
    let mut process_vector: Vec<ProcessHarvest> = Vec::new();

    if cfg!(target_os = "linux") {
        let ps_result = Command::new("ps")
            .args(&["-axo", "pid:10,comm:50,%mem:5", "--noheader"])
            .output()?;
        let ps_stdout = String::from_utf8_lossy(&ps_result.stdout);
        let split_string = ps_stdout.split('\n');
        let cpu_calc = cpu_usage_calculation(prev_idle, prev_non_idle);
        if let Ok((cpu_usage, cpu_fraction)) = cpu_calc {
            let process_stream = split_string.collect::<Vec<&str>>();

            let mut new_pid_stats: HashMap<String, (f64, Instant), RandomState> = HashMap::new();

            for process in process_stream {
                if let Ok(process_object) = convert_ps(
                    process,
                    cpu_usage,
                    cpu_fraction,
                    &prev_pid_stats,
                    &mut new_pid_stats,
                    use_current_cpu_total,
                    curr_time,
                ) {
                    if !process_object.name.is_empty() {
                        process_vector.push(process_object);
                    }
                }
            }

            *prev_pid_stats = new_pid_stats;
        }
    } else {
        let process_hashmap = sys.get_processes();
        let cpu_usage = sys.get_global_processor_info().get_cpu_usage() as f64 / 100.0;
        let num_cpus = sys.get_processors().len() as f64;
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

            let pcu = if cfg!(target_os = "windows") {
                process_val.cpu_usage() as f64
            } else {
                process_val.cpu_usage() as f64 / num_cpus
            };
            let process_cpu_usage = if use_current_cpu_total {
                pcu / cpu_usage
            } else {
                pcu
            };

            process_vector.push(ProcessHarvest {
                pid: process_val.pid() as u32,
                name,
                mem_usage_percent: process_val.memory() as f64 * 100.0 / mem_total_kb as f64,
                cpu_usage_percent: process_cpu_usage,
            });
        }
    }

    Ok(process_vector)
}
