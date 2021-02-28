use crate::Pid;
use std::path::PathBuf;
use sysinfo::ProcessStatus;

#[cfg(target_os = "linux")]
use crate::utils::error::{self, BottomError};

#[cfg(target_os = "linux")]
use fnv::{FnvHashMap, FnvHashSet};

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

    // TODO: Add real user ID
    // pub real_uid: Option<u32>,
    #[cfg(target_family = "unix")]
    pub gid: Option<libc::gid_t>,
}

#[derive(Debug, Default, Clone)]
pub struct PrevProcDetails {
    pub total_read_bytes: u64,
    pub total_write_bytes: u64,
    pub cpu_time: f64,
    pub proc_stat_path: PathBuf,
    pub proc_status_path: PathBuf,
    // pub proc_statm_path: PathBuf,
    // pub proc_exe_path: PathBuf,
    pub proc_io_path: PathBuf,
    pub proc_cmdline_path: PathBuf,
    pub just_read: bool,
}

impl PrevProcDetails {
    pub fn new(pid: Pid) -> Self {
        PrevProcDetails {
            proc_io_path: PathBuf::from(format!("/proc/{}/io", pid)),
            // proc_exe_path: PathBuf::from(format!("/proc/{}/exe", pid)),
            proc_stat_path: PathBuf::from(format!("/proc/{}/stat", pid)),
            proc_status_path: PathBuf::from(format!("/proc/{}/status", pid)),
            // proc_statm_path: PathBuf::from(format!("/proc/{}/statm", pid)),
            proc_cmdline_path: PathBuf::from(format!("/proc/{}/cmdline", pid)),
            ..PrevProcDetails::default()
        }
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
            let passwd = unsafe { libc::getpwuid(uid) };
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
fn get_linux_process_vsize_rss(stat: &[&str]) -> (u64, u64) {
    // Represents vsize and rss (bytes and page numbers respectively)
    (
        stat[20].parse::<u64>().unwrap_or(0),
        stat[21].parse::<u64>().unwrap_or(0),
    )
}

#[cfg(target_os = "linux")]
/// Preferably use this only on small files.
fn read_path_contents(path: &PathBuf) -> std::io::Result<String> {
    Ok(std::fs::read_to_string(path)?)
}

#[cfg(target_os = "linux")]
fn get_linux_process_state(stat: &[&str]) -> (char, String) {
    // The -2 offset is because of us cutting off name + pid, normally it's 2
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

#[cfg(target_os = "macos")]
fn get_macos_cpu_usage(pids: &[i32]) -> std::io::Result<std::collections::HashMap<i32, f64>> {
    use itertools::Itertools;
    let output = std::process::Command::new("ps")
        .args(&["-o", "pid=,pcpu=", "-p"])
        .arg(
            pids.iter()
                .map(i32::to_string)
                .intersperse(",".to_string())
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

#[cfg(target_os = "linux")]
fn get_uid_and_gid(path: &PathBuf) -> (Option<u32>, Option<u32>) {
    // FIXME: [OPT] - can we merge our /stat and /status calls?
    use std::io::prelude::*;
    use std::io::BufReader;

    if let Ok(file) = std::fs::File::open(path) {
        let reader = BufReader::new(file);
        let mut lines = reader.lines().skip(8);

        let (_real_uid, effective_uid) = if let Some(Ok(read_uid_line)) = lines.next() {
            let mut split_whitespace = read_uid_line.split_whitespace().skip(1);
            (
                split_whitespace.next().and_then(|x| x.parse::<u32>().ok()),
                split_whitespace.next().and_then(|x| x.parse::<u32>().ok()),
            )
        } else {
            (None, None)
        };

        let (_real_gid, effective_gid) = if let Some(Ok(read_gid_line)) = lines.next() {
            let mut split_whitespace = read_gid_line.split_whitespace().skip(1);
            (
                split_whitespace.next().and_then(|x| x.parse::<u32>().ok()),
                split_whitespace.next().and_then(|x| x.parse::<u32>().ok()),
            )
        } else {
            (None, None)
        };

        (effective_uid, effective_gid)
    } else {
        (None, None)
    }
}

#[allow(clippy::too_many_arguments)]
#[cfg(target_os = "linux")]
fn read_proc(
    pid: Pid, cpu_usage: f64, cpu_fraction: f64,
    pid_mapping: &mut FnvHashMap<Pid, PrevProcDetails>, use_current_cpu_total: bool,
    time_difference_in_secs: u64, mem_total_kb: u64, page_file_kb: u64,
) -> error::Result<ProcessHarvest> {
    use std::io::prelude::*;
    use std::io::BufReader;

    let pid_stat = pid_mapping
        .entry(pid)
        .or_insert_with(|| PrevProcDetails::new(pid));
    let stat_results = read_path_contents(&pid_stat.proc_stat_path)?;

    // truncated_name may potentially be cut!  Hence why we do the bit of code after...
    let truncated_name = stat_results
        .splitn(2, '(')
        .collect::<Vec<_>>()
        .last()
        .ok_or(BottomError::MinorError)?
        .rsplitn(2, ')')
        .collect::<Vec<_>>()
        .last()
        .ok_or(BottomError::MinorError)?
        .to_string();
    let (command, name) = {
        let cmd = read_path_contents(&pid_stat.proc_cmdline_path)?;
        let trimmed_cmd = cmd.trim();
        if trimmed_cmd.is_empty() {
            (format!("[{}]", truncated_name), truncated_name)
        } else {
            // We split by spaces and null terminators.
            let separated_strings = trimmed_cmd
                .split_terminator(|c| c == '\0' || c == ' ')
                .collect::<Vec<&str>>();

            (
                separated_strings.join(" "),
                if truncated_name.len() >= MAX_STAT_NAME_LEN {
                    if let Some(first_part) = separated_strings.first() {
                        // We're only interested in the executable part... not the file path.
                        // That's for command.
                        first_part
                            .split('/')
                            .collect::<Vec<_>>()
                            .last()
                            .unwrap_or(&truncated_name.as_str())
                            .to_string()
                    } else {
                        truncated_name
                    }
                } else {
                    truncated_name
                },
            )
        }
    };
    let stat = stat_results
        .split(')')
        .collect::<Vec<_>>()
        .last()
        .ok_or(BottomError::MinorError)?
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
    let parent_pid = stat[1].parse::<Pid>().ok();
    let (_vsize, rss) = get_linux_process_vsize_rss(&stat);
    let mem_usage_kb = rss * page_file_kb;
    let mem_usage_percent = mem_usage_kb as f64 / mem_total_kb as f64 * 100.0;
    let mem_usage_bytes = mem_usage_kb * 1024;

    // This can fail if permission is denied!

    let (total_read_bytes, total_write_bytes, read_bytes_per_sec, write_bytes_per_sec) =
        if let Ok(file) = std::fs::File::open(&pid_stat.proc_io_path) {
            let reader = BufReader::new(file);
            let mut lines = reader.lines().skip(4);

            // Represents read_bytes and write_bytes, at the 5th and 6th lines (1-index, not 0-index)
            let total_read_bytes = if let Some(Ok(read_bytes_line)) = lines.next() {
                if let Some(read_bytes) = read_bytes_line.split_whitespace().last() {
                    read_bytes.parse::<u64>().unwrap_or(0)
                } else {
                    0
                }
            } else {
                0
            };

            let total_write_bytes = if let Some(Ok(write_bytes_line)) = lines.next() {
                if let Some(write_bytes) = write_bytes_line.split_whitespace().last() {
                    write_bytes.parse::<u64>().unwrap_or(0)
                } else {
                    0
                }
            } else {
                0
            };

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

    let (uid, gid) = get_uid_and_gid(&pid_stat.proc_status_path);

    Ok(ProcessHarvest {
        pid,
        parent_pid,
        name,
        command,
        mem_usage_percent,
        mem_usage_bytes,
        cpu_usage_percent,
        total_read_bytes,
        total_write_bytes,
        read_bytes_per_sec,
        write_bytes_per_sec,
        process_state,
        process_state_char,
        uid,
        gid,
    })
}

#[cfg(target_os = "linux")]
pub fn get_process_data(
    prev_idle: &mut f64, prev_non_idle: &mut f64,
    pid_mapping: &mut FnvHashMap<Pid, PrevProcDetails>, use_current_cpu_total: bool,
    time_difference_in_secs: u64, mem_total_kb: u64, page_file_kb: u64,
) -> crate::utils::error::Result<Vec<ProcessHarvest>> {
    // TODO: [PROC THREADS] Add threads

    if let Ok((cpu_usage, cpu_fraction)) = cpu_usage_calculation(prev_idle, prev_non_idle) {
        let mut pids_to_clear: FnvHashSet<Pid> = pid_mapping.keys().cloned().collect();
        let process_vector: Vec<ProcessHarvest> = std::fs::read_dir("/proc")?
            .filter_map(|dir| {
                if let Ok(dir) = dir {
                    let pid = dir.file_name().to_string_lossy().trim().parse::<Pid>();
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
                            pids_to_clear.remove(&pid);
                            return Some(process_object);
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
                process_state: process_val.status().to_string().to_string(),
                process_state_char: convert_process_status_to_char(process_val.status()),
                uid: Some(process_val.uid),
                gid: Some(process_val.gid),
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
                process_state: process_val.status().to_string().to_string(),
                process_state_char: convert_process_status_to_char(process_val.status()),
            });
        }
    }

    #[cfg(target_os = "macos")]
    {
        let unknown_state = ProcessStatus::Unknown(0).to_string().to_string();
        let cpu_usage_unknown_pids: Vec<i32> = process_vector
            .iter()
            .filter(|process| process.process_state == unknown_state)
            .map(|process| process.pid)
            .collect();
        let cpu_usages = get_macos_cpu_usage(&cpu_usage_unknown_pids)?;
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
