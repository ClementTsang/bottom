use std::path::PathBuf;
use sysinfo::ProcessStatus;

#[cfg(target_os = "linux")]
use crate::utils::error;
#[cfg(target_os = "linux")]
use std::{
    collections::{hash_map::RandomState, HashMap},
    process::Command,
};

#[cfg(not(target_os = "linux"))]
use sysinfo::{ProcessExt, ProcessorExt, System, SystemExt};

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
    pub read_bytes_per_sec: u64,
    pub write_bytes_per_sec: u64,
    pub total_read_bytes: u64,
    pub total_write_bytes: u64,
    pub process_state: String,
    pub process_state_char: char,
}

#[derive(Debug, Default, Clone)]
pub struct PrevProcDetails {
    pub total_read_bytes: u64,
    pub total_write_bytes: u64,
    pub cpu_time: f64,
    pub proc_stat_path: PathBuf,
    pub proc_io_path: PathBuf,
}

impl PrevProcDetails {
    pub fn new(pid: u32) -> Self {
        let pid_string = pid.to_string();
        PrevProcDetails {
            proc_io_path: PathBuf::from(format!("/proc/{}/io", pid_string)),
            proc_stat_path: PathBuf::from(format!("/proc/{}/stat", pid_string)),
            ..PrevProcDetails::default()
        }
    }
}

#[cfg(target_os = "linux")]
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

#[cfg(target_os = "linux")]
fn get_process_io(path: &PathBuf) -> std::io::Result<String> {
    Ok(std::fs::read_to_string(path)?)
}

#[cfg(target_os = "linux")]
fn get_linux_process_io_usage(io_stats: &[&str]) -> (u64, u64) {
    // Represents read_bytes and write_bytes
    (
        io_stats[4].parse::<u64>().unwrap_or(0),
        io_stats[5].parse::<u64>().unwrap_or(0),
    )
}

#[cfg(target_os = "linux")]
fn get_process_stats(path: &PathBuf) -> std::io::Result<String> {
    Ok(std::fs::read_to_string(path)?)
}

#[cfg(target_os = "linux")]
fn get_linux_process_state(proc_stats: &[&str]) -> (char, String) {
    if let Some(first_char) = proc_stats[2].chars().collect::<Vec<char>>().first() {
        (
            *first_char,
            ProcessStatus::from(*first_char).to_string().to_string(),
        )
    } else {
        ('?', String::default())
    }
}

/// Note that cpu_fraction should be represented WITHOUT the x100 factor!
#[cfg(target_os = "linux")]
fn get_linux_cpu_usage(
    proc_stats: &[&str], cpu_usage: f64, cpu_fraction: f64, before_proc_val: f64,
    use_current_cpu_total: bool,
) -> std::io::Result<(f64, f64)> {
    fn get_process_cpu_stats(stats: &[&str]) -> f64 {
        // utime + stime (matches top)
        stats[13].parse::<f64>().unwrap_or(0_f64) + stats[14].parse::<f64>().unwrap_or(0_f64)
    }

    // Based heavily on https://stackoverflow.com/a/23376195 and https://stackoverflow.com/a/1424556
    let after_proc_val = get_process_cpu_stats(&proc_stats);

    if use_current_cpu_total {
        Ok((
            (after_proc_val - before_proc_val) / cpu_usage * 100_f64,
            after_proc_val,
        ))
    } else {
        Ok((
            (after_proc_val - before_proc_val) / cpu_usage * 100_f64 * cpu_fraction,
            after_proc_val,
        ))
    }
}

#[cfg(target_os = "linux")]
fn convert_ps<S: core::hash::BuildHasher>(
    process: &str, cpu_usage: f64, cpu_fraction: f64,
    prev_pid_stats: &mut HashMap<u32, PrevProcDetails, S>,
    new_pid_stats: &mut HashMap<u32, PrevProcDetails, S>, use_current_cpu_total: bool,
    time_difference_in_secs: u64,
) -> std::io::Result<ProcessHarvest> {
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

    let mut new_pid_stat = if let Some(prev_proc_stats) = prev_pid_stats.remove(&pid) {
        prev_proc_stats
    } else {
        PrevProcDetails::new(pid)
    };

    let (cpu_usage_percent, process_state_char, process_state) =
        if let Ok(stat_results) = get_process_stats(&new_pid_stat.proc_stat_path) {
            let proc_stats = stat_results.split_whitespace().collect::<Vec<&str>>();
            let (process_state_char, process_state) = get_linux_process_state(&proc_stats);

            let (cpu_usage_percent, after_proc_val) = get_linux_cpu_usage(
                &proc_stats,
                cpu_usage,
                cpu_fraction,
                new_pid_stat.cpu_time,
                use_current_cpu_total,
            )?;
            new_pid_stat.cpu_time = after_proc_val;

            (cpu_usage_percent, process_state_char, process_state)
        } else {
            (0.0, '?', String::new())
        };

    // This can fail if permission is denied!
    let (total_read_bytes, total_write_bytes, read_bytes_per_sec, write_bytes_per_sec) =
        if let Ok(io_results) = get_process_io(&new_pid_stat.proc_io_path) {
            let io_stats = io_results.split_whitespace().collect::<Vec<&str>>();

            let (total_read_bytes, total_write_bytes) = get_linux_process_io_usage(&io_stats);
            let read_bytes_per_sec = if time_difference_in_secs == 0 {
                0
            } else {
                (total_write_bytes - new_pid_stat.total_write_bytes) / time_difference_in_secs
            };
            let write_bytes_per_sec = if time_difference_in_secs == 0 {
                0
            } else {
                (total_read_bytes - new_pid_stat.total_read_bytes) / time_difference_in_secs
            };

            new_pid_stat.total_read_bytes = total_read_bytes;
            new_pid_stat.total_write_bytes = total_write_bytes;

            (
                total_read_bytes,
                total_write_bytes,
                read_bytes_per_sec,
                write_bytes_per_sec,
            )
        } else {
            (0, 0, 0, 0)
        };

    new_pid_stats.insert(pid, new_pid_stat);

    Ok(ProcessHarvest {
        pid,
        name,
        mem_usage_percent,
        cpu_usage_percent,
        total_read_bytes,
        total_write_bytes,
        read_bytes_per_sec,
        write_bytes_per_sec,
        process_state,
        process_state_char,
    })
}

#[cfg(target_os = "linux")]
pub fn linux_get_processes_list(
    prev_idle: &mut f64, prev_non_idle: &mut f64,
    prev_pid_stats: &mut HashMap<u32, PrevProcDetails, RandomState>, use_current_cpu_total: bool,
    time_difference_in_secs: u64,
) -> crate::utils::error::Result<Vec<ProcessHarvest>> {
    let ps_result = Command::new("ps")
        .args(&["-axo", "pid:10,comm:50,%mem:5", "--noheader"])
        .output()?;
    let ps_stdout = String::from_utf8_lossy(&ps_result.stdout);
    let split_string = ps_stdout.split('\n');
    let cpu_calc = cpu_usage_calculation(prev_idle, prev_non_idle);
    if let Ok((cpu_usage, cpu_fraction)) = cpu_calc {
        let process_list = split_string.collect::<Vec<&str>>();

        let mut new_pid_stats = HashMap::new();

        let process_vector: Vec<ProcessHarvest> = process_list
            .iter()
            .filter_map(|process| {
                if process.trim().is_empty() {
                    None
                } else if let Ok(process_object) = convert_ps(
                    process,
                    cpu_usage,
                    cpu_fraction,
                    prev_pid_stats,
                    &mut new_pid_stats,
                    use_current_cpu_total,
                    time_difference_in_secs,
                ) {
                    if !process_object.name.is_empty() {
                        Some(process_object)
                    } else {
                        None
                    }
                } else {
                    None
                }
            })
            .collect();

        *prev_pid_stats = new_pid_stats;
        Ok(process_vector)
    } else {
        Ok(Vec::new())
    }
}

#[cfg(not(target_os = "linux"))]
pub fn windows_macos_get_processes_list(
    sys: &System, use_current_cpu_total: bool, mem_total_kb: u64,
) -> crate::utils::error::Result<Vec<ProcessHarvest>> {
    let mut process_vector: Vec<ProcessHarvest> = Vec::new();

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

        let disk_usage = process_val.disk_usage();

        process_vector.push(ProcessHarvest {
            pid: process_val.pid() as u32,
            name,
            mem_usage_percent: process_val.memory() as f64 * 100.0 / mem_total_kb as f64,
            cpu_usage_percent: process_cpu_usage,
            read_bytes_per_sec: disk_usage.read_bytes,
            write_bytes_per_sec: disk_usage.written_bytes,
            total_read_bytes: disk_usage.total_read_bytes,
            total_write_bytes: disk_usage.total_written_bytes,
            process_state: process_val.status().to_string().to_string(),
            process_state_char: convert_process_status_to_char(process_val.status()),
        });
    }

    Ok(process_vector)
}

#[allow(unused_variables)]
#[cfg(not(target_os = "linux"))]
fn convert_process_status_to_char(status: ProcessStatus) -> char {
    if cfg!(target_os = "macos") {
        #[cfg(target_os = "macos")]
        {
            match status {
                ProcessStatus::Run => 'R',
                ProcessStatus::Sleep => 'S',
                ProcessStatus::Idle => 'D',
                ProcessStatus::Zombie => 'Z',
                ProcessStatus::Dead => 'X',
                _ => '?',
            }
        }
        #[cfg(not(target_os = "macos"))]
        {
            '?'
        }
    } else {
        'R'
    }
}
