use crate::Pid;

use sysinfo::ProcessStatus;

#[cfg(target_family = "unix")]
use crate::utils::error;

#[cfg(target_os = "linux")]
use procfs::process::{Process, Stat};

#[cfg(target_os = "linux")]
use crate::utils::error::BottomError;

#[cfg(target_os = "linux")]
use fxhash::{FxHashMap, FxHashSet};

#[cfg(not(target_os = "linux"))]
use sysinfo::{ProcessExt, ProcessorExt, System, SystemExt};

/// Maximum character length of a /proc/<PID>/stat process name that we'll accept.
#[cfg(target_os = "linux")]
const MAX_STAT_NAME_LEN: usize = 15;

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
    User,
    Count,
}

impl std::fmt::Display for ProcessSorting {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "{}",
            match &self {
                ProcessSorting::CpuPercent => "CPU%",
                ProcessSorting::MemPercent => "Mem%",
                ProcessSorting::Mem => "Mem",
                ProcessSorting::ReadPerSecond => "R/s",
                ProcessSorting::WritePerSecond => "W/s",
                ProcessSorting::TotalRead => "T.Read",
                ProcessSorting::TotalWrite => "T.Write",
                ProcessSorting::State => "State",
                ProcessSorting::ProcessName => "Name",
                ProcessSorting::Command => "Command",
                ProcessSorting::Pid => "PID",
                ProcessSorting::Count => "Count",
                ProcessSorting::User => "User",
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
    pub pid: Pid,
    pub parent_pid: Option<Pid>, // Remember, parent_pid 0 is root...
    pub cpu_usage_percent: f64,
    pub mem_usage_percent: f64,
    pub mem_usage_bytes: u64,
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

    /// This is the *effective* user ID.
    #[cfg(target_family = "unix")]
    pub uid: Option<libc::uid_t>,
}

#[derive(Debug, Clone)]
pub struct PrevProcDetails {
    pub total_read_bytes: u64,
    pub total_write_bytes: u64,
    pub cpu_time: u64,
    pub process: Process,
}

impl PrevProcDetails {
    fn new(pid: Pid) -> error::Result<Self> {
        Ok(Self {
            total_read_bytes: 0,
            total_write_bytes: 0,
            cpu_time: 0,
            process: Process::new(pid)?,
        })
    }
}

#[cfg(target_family = "unix")]
#[derive(Debug, Default)]
pub struct UserTable {
    pub uid_user_mapping: std::collections::HashMap<libc::uid_t, String>,
}

#[cfg(target_family = "unix")]
impl UserTable {
    pub fn get_uid_to_username_mapping(&mut self, uid: libc::uid_t) -> error::Result<String> {
        if let Some(user) = self.uid_user_mapping.get(&uid) {
            Ok(user.clone())
        } else {
            // SAFETY: getpwuid returns a null pointer if no passwd entry is found for the uid
            let passwd = unsafe { libc::getpwuid(uid) };

            if passwd.is_null() {
                return Err(error::BottomError::QueryError("Missing passwd".into()));
            }

            let username = unsafe { std::ffi::CStr::from_ptr((*passwd).pw_name) }
                .to_str()?
                .to_string();
            self.uid_user_mapping.insert(uid, username.clone());

            Ok(username)
        }
    }
}

#[cfg(target_os = "linux")]
fn cpu_usage_calculation(
    prev_idle: &mut f64, prev_non_idle: &mut f64,
) -> error::Result<(f64, f64)> {
    use std::io::prelude::*;
    use std::io::BufReader;

    // From SO answer: https://stackoverflow.com/a/23376195

    let mut reader = BufReader::new(std::fs::File::open("/proc/stat")?);
    let mut first_line = String::new();
    reader.read_line(&mut first_line)?;

    let val = first_line.split_whitespace().collect::<Vec<&str>>();

    // SC in case that the parsing will fail due to length:
    if val.len() <= 10 {
        return Err(error::BottomError::InvalidIo(format!(
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

/// Returns the usage and a new set of process times. Note: cpu_fraction should be represented WITHOUT the x100 factor!
#[cfg(target_os = "linux")]
fn get_linux_cpu_usage(
    stat: &Stat, cpu_usage: f64, cpu_fraction: f64, prev_proc_times: u64,
    use_current_cpu_total: bool,
) -> (f64, u64) {
    // Based heavily on https://stackoverflow.com/a/23376195 and https://stackoverflow.com/a/1424556
    let new_proc_times = stat.utime + stat.stime;
    let diff = (new_proc_times - prev_proc_times) as f64; // I HATE that it's done like this but there isn't a try_from for u64 -> f64... we can accept a bit of loss in the worst case though

    if cpu_usage == 0.0 {
        (0.0, new_proc_times)
    } else if use_current_cpu_total {
        (diff / cpu_usage * 100_f64, new_proc_times)
    } else {
        (diff / cpu_usage * 100_f64 * cpu_fraction, new_proc_times)
    }
}

#[cfg(target_os = "macos")]
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

#[allow(clippy::too_many_arguments)]
#[cfg(target_os = "linux")]
fn read_proc(
    prev_proc: &PrevProcDetails, stat: &Stat, cpu_usage: f64, cpu_fraction: f64,
    use_current_cpu_total: bool, time_difference_in_secs: u64, mem_total_kb: u64,
) -> error::Result<(ProcessHarvest, u64)> {
    use std::convert::TryFrom;

    let process = &prev_proc.process;

    let (command, name) = {
        let truncated_name = stat.comm.as_str();
        if let Ok(cmdline) = process.cmdline() {
            if cmdline.is_empty() {
                (format!("[{}]", truncated_name), truncated_name.to_string())
            } else {
                (
                    cmdline.join(" "),
                    if truncated_name.len() >= MAX_STAT_NAME_LEN {
                        if let Some(first_part) = cmdline.first() {
                            // We're only interested in the executable part... not the file path.
                            // That's for command.
                            first_part
                                .split('/')
                                .collect::<Vec<_>>()
                                .last()
                                .unwrap_or(&truncated_name)
                                .to_string()
                        } else {
                            truncated_name.to_string()
                        }
                    } else {
                        truncated_name.to_string()
                    },
                )
            }
        } else {
            (truncated_name.to_string(), truncated_name.to_string())
        }
    };

    let process_state_char = stat.state;
    let process_state = ProcessStatus::from(process_state_char).to_string();
    let (cpu_usage_percent, new_process_times) = get_linux_cpu_usage(
        &stat,
        cpu_usage,
        cpu_fraction,
        prev_proc.cpu_time,
        use_current_cpu_total,
    );
    let parent_pid = Some(stat.ppid);
    let mem_usage_bytes = u64::try_from(stat.rss_bytes()).unwrap_or(0);
    let mem_usage_kb = mem_usage_bytes / 1024;
    let mem_usage_percent = mem_usage_kb as f64 / mem_total_kb as f64 * 100.0;

    // This can fail if permission is denied!

    let (total_read_bytes, total_write_bytes, read_bytes_per_sec, write_bytes_per_sec) =
        if let Ok(io) = process.io() {
            let total_read_bytes = io.read_bytes;
            let total_write_bytes = io.write_bytes;

            let read_bytes_per_sec = if time_difference_in_secs == 0 {
                0
            } else {
                total_read_bytes.saturating_sub(prev_proc.total_read_bytes)
                    / time_difference_in_secs
            };
            let write_bytes_per_sec = if time_difference_in_secs == 0 {
                0
            } else {
                total_write_bytes.saturating_sub(prev_proc.total_write_bytes)
                    / time_difference_in_secs
            };

            (
                total_read_bytes,
                total_write_bytes,
                read_bytes_per_sec,
                write_bytes_per_sec,
            )
        } else {
            (0, 0, 0, 0)
        };

    let uid = Some(process.owner);

    Ok((
        ProcessHarvest {
            pid: process.pid,
            parent_pid,
            cpu_usage_percent,
            mem_usage_percent,
            mem_usage_bytes,
            name,
            command,
            read_bytes_per_sec,
            write_bytes_per_sec,
            total_read_bytes,
            total_write_bytes,
            process_state,
            process_state_char,
            uid,
        },
        new_process_times,
    ))
}

#[cfg(target_os = "linux")]
pub fn get_process_data(
    prev_idle: &mut f64, prev_non_idle: &mut f64,
    pid_mapping: &mut FxHashMap<Pid, PrevProcDetails>, use_current_cpu_total: bool,
    time_difference_in_secs: u64, mem_total_kb: u64,
) -> crate::utils::error::Result<Vec<ProcessHarvest>> {
    // TODO: [PROC THREADS] Add threads

    if let Ok((cpu_usage, cpu_fraction)) = cpu_usage_calculation(prev_idle, prev_non_idle) {
        let mut pids_to_clear: FxHashSet<Pid> = pid_mapping.keys().cloned().collect();

        let process_vector: Vec<ProcessHarvest> = std::fs::read_dir("/proc")?
            .filter_map(|dir| {
                if let Ok(dir) = dir {
                    if let Ok(pid) = dir.file_name().to_string_lossy().trim().parse::<Pid>() {
                        let mut fresh = false;
                        if !pid_mapping.contains_key(&pid) {
                            if let Ok(ppd) = PrevProcDetails::new(pid) {
                                pid_mapping.insert(pid, ppd);
                                fresh = true;
                            } else {
                                // Bail early.
                                return None;
                            }
                        };

                        if let Some(prev_proc_details) = pid_mapping.get_mut(&pid) {
                            let stat;
                            let stat_live;
                            if fresh {
                                stat = &prev_proc_details.process.stat;
                            } else if let Ok(s) = prev_proc_details.process.stat() {
                                stat_live = s;
                                stat = &stat_live;
                            } else {
                                // Bail early.
                                return None;
                            }

                            if let Ok((process_harvest, new_process_times)) = read_proc(
                                &prev_proc_details,
                                stat,
                                cpu_usage,
                                cpu_fraction,
                                use_current_cpu_total,
                                time_difference_in_secs,
                                mem_total_kb,
                            ) {
                                prev_proc_details.cpu_time = new_process_times;
                                prev_proc_details.total_read_bytes =
                                    process_harvest.total_read_bytes;
                                prev_proc_details.total_write_bytes =
                                    process_harvest.total_write_bytes;

                                pids_to_clear.remove(&pid);
                                return Some(process_harvest);
                            }
                        }
                    }
                }

                None
            })
            .collect();

        pids_to_clear.iter().for_each(|pid| {
            pid_mapping.remove(pid);
        });

        Ok(process_vector)
    } else {
        Err(BottomError::GenericError(
            "Could not calculate CPU usage.".to_string(),
        ))
    }
}

#[cfg(not(target_os = "linux"))]
pub fn get_process_data(
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
        let command = {
            let command = process_val.cmd().join(" ");
            if command.is_empty() {
                name.to_string()
            } else {
                command
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
        #[cfg(target_os = "macos")]
        {
            process_vector.push(ProcessHarvest {
                pid: process_val.pid(),
                parent_pid: process_val.parent(),
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
        #[cfg(not(target_os = "macos"))]
        {
            process_vector.push(ProcessHarvest {
                pid: process_val.pid(),
                parent_pid: process_val.parent(),
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
            });
        }
    }

    #[cfg(target_os = "macos")]
    {
        let unknown_state = ProcessStatus::Unknown(0).to_string();
        let cpu_usage_unknown_pids: Vec<i32> = process_vector
            .iter()
            .filter(|process| process.process_state == unknown_state)
            .map(|process| process.pid)
            .collect();
        let cpu_usages = get_macos_process_cpu_usage(&cpu_usage_unknown_pids)?;
        for process in &mut process_vector {
            if cpu_usages.contains_key(&process.pid) {
                process.cpu_usage_percent = if num_cpus == 0.0 {
                    *cpu_usages.get(&process.pid).unwrap()
                } else {
                    *cpu_usages.get(&process.pid).unwrap() / num_cpus
                };
            }
        }
    }

    Ok(process_vector)
}

#[allow(unused_variables)]
#[cfg(not(target_os = "linux"))]
fn convert_process_status_to_char(status: ProcessStatus) -> char {
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
        'R'
    }
}
