use std::path::PathBuf;
use sysinfo::ProcessStatus;

#[cfg(target_os = "linux")]
use std::collections::{hash_map::RandomState, HashMap};

#[cfg(not(target_os = "linux"))]
use sysinfo::{ProcessExt, ProcessorExt, System, SystemExt};

use crate::utils::error::{self, BottomError};

// TODO: Add value so we know if it's sorted ascending or descending by default?
#[derive(Clone, PartialEq, Eq, Hash, Debug)]
pub enum ProcessSorting {
    CpuPercent,
    Mem,
    MemPercent,
    Pid,
    ProcessName,
    Command,
    ReadPerSecond,
    WritePerSecond,
    TotalRead,
    TotalWrite,
    State,
}

impl std::fmt::Display for ProcessSorting {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        use ProcessSorting::*;
        write!(
            f,
            "{}",
            match &self {
                CpuPercent => "CPU%",
                MemPercent => "Mem%",
                Mem => "Mem",
                ReadPerSecond => "R/s",
                WritePerSecond => "W/s",
                TotalRead => "Read",
                TotalWrite => "Write",
                State => "State",
                ProcessName => "Name",
                Command => "Command",
                Pid => "PID",
            }
        )
    }
}

impl Default for ProcessSorting {
    fn default() -> Self {
        ProcessSorting::CpuPercent
    }
}

#[derive(Debug, Clone, Default)]
pub struct ProcessHarvest {
    pub pid: u32,
    pub cpu_usage_percent: f64,
    pub mem_usage_percent: f64,
    pub mem_usage_kb: u64,
    // pub rss_kb: u64,
    // pub virt_kb: u64,
    pub name: String,
    pub command: String,
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
    pub proc_exe_path: PathBuf,
    pub proc_io_path: PathBuf,
    pub proc_cmdline_path: PathBuf,
    pub just_read: bool,
}

impl PrevProcDetails {
    pub fn new(pid: u32) -> Self {
        PrevProcDetails {
            proc_io_path: PathBuf::from(format!("/proc/{}/io", pid)),
            proc_exe_path: PathBuf::from(format!("/proc/{}/exe", pid)),
            proc_stat_path: PathBuf::from(format!("/proc/{}/stat", pid)),
            proc_cmdline_path: PathBuf::from(format!("/proc/{}/cmdline", pid)),
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
fn get_linux_process_io_usage(stat: &[&str]) -> (u64, u64) {
    // Represents read_bytes and write_bytes
    (
        stat[9].parse::<u64>().unwrap_or(0),
        stat[11].parse::<u64>().unwrap_or(0),
    )
}

#[cfg(target_os = "linux")]
fn get_linux_process_vsize_rss(stat: &[&str]) -> (u64, u64) {
    // Represents vsize and rss (bytes and page numbers respectively)
    (
        stat[20].parse::<u64>().unwrap_or(0),
        stat[21].parse::<u64>().unwrap_or(0),
    )
}

#[cfg(target_os = "linux")]
fn read_path_contents(path: &PathBuf) -> std::io::Result<String> {
    Ok(std::fs::read_to_string(path)?)
}

#[cfg(target_os = "linux")]
fn get_linux_process_state(stat: &[&str]) -> (char, String) {
    // The -2 offset is because of us cutting off name + pid
    if let Some(first_char) = stat[0].chars().collect::<Vec<char>>().first() {
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
    proc_stats: &[&str], cpu_usage: f64, cpu_fraction: f64, prev_proc_val: &mut f64,
    use_current_cpu_total: bool,
) -> std::io::Result<f64> {
    fn get_process_cpu_stats(stat: &[&str]) -> f64 {
        // utime + stime (matches top), the -2 offset is because of us cutting off name + pid (normally 13, 14)
        stat[11].parse::<f64>().unwrap_or(0_f64) + stat[12].parse::<f64>().unwrap_or(0_f64)
    }

    // Based heavily on https://stackoverflow.com/a/23376195 and https://stackoverflow.com/a/1424556
    let new_proc_val = get_process_cpu_stats(&proc_stats);

    if cpu_usage == 0.0 {
        Ok(0_f64)
    } else if use_current_cpu_total {
        let res = Ok((new_proc_val - *prev_proc_val) / cpu_usage * 100_f64);
        *prev_proc_val = new_proc_val;
        res
    } else {
        let res = Ok((new_proc_val - *prev_proc_val) / cpu_usage * 100_f64 * cpu_fraction);
        *prev_proc_val = new_proc_val;
        res
    }
}

#[allow(clippy::too_many_arguments)]
#[cfg(target_os = "linux")]
fn read_proc<S: core::hash::BuildHasher>(
    pid: u32, cpu_usage: f64, cpu_fraction: f64,
    pid_mapping: &mut HashMap<u32, PrevProcDetails, S>, use_current_cpu_total: bool,
    time_difference_in_secs: u64, mem_total_kb: u64, page_file_kb: u64,
) -> error::Result<ProcessHarvest> {
    let pid_stat = pid_mapping
        .entry(pid)
        .or_insert_with(|| PrevProcDetails::new(pid));
    let stat_results = read_path_contents(&pid_stat.proc_stat_path)?;
    let name = stat_results
        .splitn(2, '(')
        .collect::<Vec<_>>()
        .last()
        .ok_or(BottomError::MinorError())?
        .rsplitn(2, ')')
        .collect::<Vec<_>>()
        .last()
        .ok_or(BottomError::MinorError())?
        .to_string();
    let command = {
        let cmd = read_path_contents(&pid_stat.proc_cmdline_path)?;
        if cmd.trim().is_empty() {
            format!("[{}]", name)
        } else {
            cmd
        }
    };
    let stat = stat_results
        .split(')')
        .collect::<Vec<_>>()
        .last()
        .ok_or(BottomError::MinorError())?
        .split_whitespace()
        .collect::<Vec<&str>>();
    let (process_state_char, process_state) = get_linux_process_state(&stat);
    let cpu_usage_percent = get_linux_cpu_usage(
        &stat,
        cpu_usage,
        cpu_fraction,
        &mut pid_stat.cpu_time,
        use_current_cpu_total,
    )?;
    let (_vsize, rss) = get_linux_process_vsize_rss(&stat);
    let mem_usage_kb = rss * page_file_kb;
    let mem_usage_percent = mem_usage_kb as f64 * 100.0 / mem_total_kb as f64;

    // This can fail if permission is denied!
    let (total_read_bytes, total_write_bytes, read_bytes_per_sec, write_bytes_per_sec) =
        if let Ok(io_results) = get_process_io(&pid_stat.proc_io_path) {
            let io_stats = io_results.split_whitespace().collect::<Vec<&str>>();

            let (total_read_bytes, total_write_bytes) = get_linux_process_io_usage(&io_stats);
            let read_bytes_per_sec = if time_difference_in_secs == 0 {
                0
            } else {
                total_read_bytes.saturating_sub(pid_stat.total_read_bytes) / time_difference_in_secs
            };
            let write_bytes_per_sec = if time_difference_in_secs == 0 {
                0
            } else {
                total_write_bytes.saturating_sub(pid_stat.total_write_bytes)
                    / time_difference_in_secs
            };

            pid_stat.total_read_bytes = total_read_bytes;
            pid_stat.total_write_bytes = total_write_bytes;

            (
                total_read_bytes,
                total_write_bytes,
                read_bytes_per_sec,
                write_bytes_per_sec,
            )
        } else {
            (0, 0, 0, 0)
        };

    Ok(ProcessHarvest {
        pid,
        name,
        command,
        mem_usage_percent,
        mem_usage_kb,
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
    pid_mapping: &mut HashMap<u32, PrevProcDetails, RandomState>, use_current_cpu_total: bool,
    time_difference_in_secs: u64, mem_total_kb: u64, page_file_kb: u64,
) -> crate::utils::error::Result<Vec<ProcessHarvest>> {
    if let Ok((cpu_usage, cpu_fraction)) = cpu_usage_calculation(prev_idle, prev_non_idle) {
        let process_vector: Vec<ProcessHarvest> = std::fs::read_dir("/proc")?
            .filter_map(|dir| {
                if let Ok(dir) = dir {
                    let pid = dir.file_name().to_string_lossy().trim().parse::<u32>();
                    if let Ok(pid) = pid {
                        // I skip checking if the path is also a directory, it's not needed I think?
                        if let Ok(process_object) = read_proc(
                            pid,
                            cpu_usage,
                            cpu_fraction,
                            pid_mapping,
                            use_current_cpu_total,
                            time_difference_in_secs,
                            mem_total_kb,
                            page_file_kb,
                        ) {
                            return Some(process_object);
                        }
                    }
                }

                None
            })
            .collect();

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
        let path = {
            let path = process_val.cmd().join(" ");
            if path.is_empty() {
                name.to_string()
            } else {
                path
            }
        };

        let pcu = if cfg!(target_os = "windows") || num_cpus == 0.0 {
            process_val.cpu_usage() as f64
        } else {
            process_val.cpu_usage() as f64 / num_cpus
        };
        let process_cpu_usage = if use_current_cpu_total && cpu_usage > 0.0 {
            pcu / cpu_usage
        } else {
            pcu
        };

        let disk_usage = process_val.disk_usage();

        process_vector.push(ProcessHarvest {
            pid: process_val.pid() as u32,
            name,
            command,
            mem_usage_percent: if mem_total_kb > 0 {
                process_val.memory() as f64 * 100.0 / mem_total_kb as f64
            } else {
                0.0
            },
            mem_usage_kb: process_val.memory(),
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
