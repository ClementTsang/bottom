//! Process data collection for Linux.

use std::collections::hash_map::Entry;

use crate::utils::error::{self, BottomError};
use crate::Pid;

use super::ProcessHarvest;

use sysinfo::ProcessStatus;

use procfs::process::{Process, Stat};

use fxhash::{FxHashMap, FxHashSet};

/// Maximum character length of a /proc/<PID>/stat process name.
/// If it's equal or greater, then we instead refer to the command for the name.
const MAX_STAT_NAME_LEN: usize = 15;

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

fn calculate_idle_values(line: String) -> (f64, f64) {
    /// Converts a `Option<&str>` value to an f64. If it fails to parse or is `None`, then it will return `0_f64`.
    fn str_to_f64(val: Option<&str>) -> f64 {
        val.and_then(|v| v.parse::<f64>().ok()).unwrap_or(0_f64)
    }

    let mut val = line.split_whitespace();
    let user = str_to_f64(val.next());
    let nice: f64 = str_to_f64(val.next());
    let system: f64 = str_to_f64(val.next());
    let idle: f64 = str_to_f64(val.next());
    let iowait: f64 = str_to_f64(val.next());
    let irq: f64 = str_to_f64(val.next());
    let softirq: f64 = str_to_f64(val.next());
    let steal: f64 = str_to_f64(val.next());

    // Note we do not get guest/guest_nice, as they are calculated as part of user/nice respectively
    // See https://github.com/htop-dev/htop/blob/main/linux/LinuxProcessList.c

    let idle = idle + iowait;
    let non_idle = user + nice + system + irq + softirq + steal;

    (idle, non_idle)
}

fn cpu_usage_calculation(
    prev_idle: &mut f64, prev_non_idle: &mut f64,
) -> error::Result<(f64, f64)> {
    use std::io::prelude::*;
    use std::io::BufReader;

    // From SO answer: https://stackoverflow.com/a/23376195
    let mut reader = BufReader::new(std::fs::File::open("/proc/stat")?);
    let mut first_line = String::new();
    reader.read_line(&mut first_line)?;

    let (idle, non_idle) = calculate_idle_values(first_line);

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

#[allow(clippy::too_many_arguments)]
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
                                .rsplit_once('/')
                                .map(|(_prefix, suffix)| suffix)
                                .unwrap_or(truncated_name)
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
        stat,
        cpu_usage,
        cpu_fraction,
        prev_proc.cpu_time,
        use_current_cpu_total,
    );
    let parent_pid = Some(stat.ppid);
    let mem_usage_bytes = u64::try_from(stat.rss_bytes()?).unwrap_or(0);
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
                        if let Entry::Vacant(entry) = pid_mapping.entry(pid) {
                            if let Ok(ppd) = PrevProcDetails::new(pid) {
                                entry.insert(ppd);
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
                                prev_proc_details,
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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_proc_cpu_parse() {
        assert_eq!(
            (100_f64, 200_f64),
            calculate_idle_values("100 0 100 100".to_string()),
            "Failed to properly calculate idle/non-idle for /proc/stat CPU with 4 values"
        );
        assert_eq!(
            (120_f64, 200_f64),
            calculate_idle_values("100 0 100 100 20".to_string()),
            "Failed to properly calculate idle/non-idle for /proc/stat CPU with 5 values"
        );
        assert_eq!(
            (120_f64, 230_f64),
            calculate_idle_values("100 0 100 100 20 30".to_string()),
            "Failed to properly calculate idle/non-idle for /proc/stat CPU with 6 values"
        );
        assert_eq!(
            (120_f64, 270_f64),
            calculate_idle_values("100 0 100 100 20 30 40".to_string()),
            "Failed to properly calculate idle/non-idle for /proc/stat CPU with 7 values"
        );
        assert_eq!(
            (120_f64, 320_f64),
            calculate_idle_values("100 0 100 100 20 30 40 50".to_string()),
            "Failed to properly calculate idle/non-idle for /proc/stat CPU with 8 values"
        );
        assert_eq!(
            (120_f64, 320_f64),
            calculate_idle_values("100 0 100 100 20 30 40 50 100".to_string()),
            "Failed to properly calculate idle/non-idle for /proc/stat CPU with 9 values"
        );
        assert_eq!(
            (120_f64, 320_f64),
            calculate_idle_values("100 0 100 100 20 30 40 50 100 200".to_string()),
            "Failed to properly calculate idle/non-idle for /proc/stat CPU with 10 values"
        );
    }
}
