//! Process data collection for Linux.

mod process;

use std::{
    fs::{self, File},
    io::{BufRead, BufReader},
    time::Duration,
};

use hashbrown::{HashMap, HashSet};
use process::*;
use sysinfo::ProcessStatus;

use crate::new_data_collection::{
    error::CollectionResult,
    sources::{common::processes::ProcessHarvest, unix::processes::user_table::UserTable, Pid},
};

/// Maximum character length of a `/proc/<PID>/stat`` process name.
/// If it's equal or greater, then we instead refer to the command for the name.
const MAX_STAT_NAME_LEN: usize = 15;

#[derive(Debug, Clone, Default)]
pub struct PrevProcDetails {
    total_read_bytes: u64,
    total_write_bytes: u64,
    cpu_time: u64,
}

/// Given `/proc/stat` file contents, determine the idle and non-idle values of
/// the CPU used to calculate CPU usage.
fn fetch_cpu_usage(line: &str) -> (f64, f64) {
    /// Converts a `Option<&str>` value to an f64. If it fails to parse or is
    /// `None`, it will return `0_f64`.
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

    // Note we do not get guest/guest_nice, as they are calculated as part of
    // user/nice respectively See https://github.com/htop-dev/htop/blob/main/linux/LinuxProcessList.c
    let idle = idle + iowait;
    let non_idle = user + nice + system + irq + softirq + steal;

    (idle, non_idle)
}

struct CpuUsage {
    /// Difference between the total delta and the idle delta.
    cpu_usage: f64,

    /// Overall CPU usage as a fraction.
    cpu_fraction: f64,
}

fn cpu_usage_calculation(
    prev_idle: &mut f64, prev_non_idle: &mut f64,
) -> CollectionResult<CpuUsage> {
    let (idle, non_idle) = {
        // From SO answer: https://stackoverflow.com/a/23376195
        let first_line = {
            // We just need a single line from this file. Read it and return it.
            let mut reader = BufReader::new(File::open("/proc/stat")?);
            let mut buffer = String::new();
            reader.read_line(&mut buffer)?;

            buffer
        };

        fetch_cpu_usage(&first_line)
    };

    let total = idle + non_idle;
    let prev_total = *prev_idle + *prev_non_idle;

    let total_delta = total - prev_total;
    let idle_delta = idle - *prev_idle;

    *prev_idle = idle;
    *prev_non_idle = non_idle;

    // TODO: Should these return errors instead?
    let cpu_usage = if total_delta - idle_delta != 0.0 {
        total_delta - idle_delta
    } else {
        1.0
    };

    let cpu_fraction = if total_delta != 0.0 {
        cpu_usage / total_delta
    } else {
        0.0
    };

    Ok(CpuUsage {
        cpu_usage,
        cpu_fraction,
    })
}

/// Returns the usage and a new set of process times.
///
/// NB: cpu_fraction should be represented WITHOUT the x100 factor!
fn get_linux_cpu_usage(
    stat: &Stat, cpu_usage: f64, cpu_fraction: f64, prev_proc_times: u64,
    use_current_cpu_total: bool,
) -> (f32, u64) {
    // Based heavily on https://stackoverflow.com/a/23376195 and https://stackoverflow.com/a/1424556
    let new_proc_times = stat.utime + stat.stime;
    let diff = (new_proc_times - prev_proc_times) as f64; // No try_from for u64 -> f64... oh well.

    if cpu_usage == 0.0 {
        (0.0, new_proc_times)
    } else if use_current_cpu_total {
        (((diff / cpu_usage) * 100.0) as f32, new_proc_times)
    } else {
        (
            ((diff / cpu_usage) * 100.0 * cpu_fraction) as f32,
            new_proc_times,
        )
    }
}

fn read_proc(
    prev_proc: &PrevProcDetails, process: Process, args: ReadProcArgs, user_table: &mut UserTable,
) -> CollectionResult<(ProcessHarvest, u64)> {
    let Process {
        pid: _,
        uid,
        stat,
        io,
        cmdline,
    } = process;

    let ReadProcArgs {
        use_current_cpu_total,
        cpu_usage,
        cpu_fraction,
        total_memory,
        time_difference_in_secs,
        uptime,
    } = args;

    let (command, name) = {
        let truncated_name = stat.comm.as_str();
        if let Ok(cmdline) = cmdline {
            if cmdline.is_empty() {
                (format!("[{truncated_name}]"), truncated_name.to_string())
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
    let process_state = (
        ProcessStatus::from(process_state_char).to_string(),
        process_state_char,
    );
    let (cpu_usage_percent, new_process_times) = get_linux_cpu_usage(
        &stat,
        cpu_usage,
        cpu_fraction,
        prev_proc.cpu_time,
        use_current_cpu_total,
    );
    let parent_pid = Some(stat.ppid);
    let mem_usage_bytes = stat.rss_bytes();
    let mem_usage_percent = (mem_usage_bytes as f64 / total_memory as f64 * 100.0) as f32;

    // This can fail if permission is denied!
    let (total_read_bytes, total_write_bytes, read_bytes_per_sec, write_bytes_per_sec) =
        if let Ok(io) = io {
            let total_read_bytes = io.read_bytes;
            let total_write_bytes = io.write_bytes;
            let prev_total_read_bytes = prev_proc.total_read_bytes;
            let prev_total_write_bytes = prev_proc.total_write_bytes;

            let read_bytes_per_sec = total_read_bytes
                .saturating_sub(prev_total_read_bytes)
                .checked_div(time_difference_in_secs)
                .unwrap_or(0);

            let write_bytes_per_sec = total_write_bytes
                .saturating_sub(prev_total_write_bytes)
                .checked_div(time_difference_in_secs)
                .unwrap_or(0);

            (
                total_read_bytes,
                total_write_bytes,
                read_bytes_per_sec,
                write_bytes_per_sec,
            )
        } else {
            (0, 0, 0, 0)
        };

    let user = uid
        .and_then(|uid| {
            user_table
                .get_uid_to_username_mapping(uid)
                .map(Into::into)
                .ok()
        })
        .unwrap_or_else(|| "N/A".into());

    let time = if let Ok(ticks_per_sec) = u32::try_from(rustix::param::clock_ticks_per_second()) {
        if ticks_per_sec == 0 {
            Duration::ZERO
        } else {
            Duration::from_secs(uptime.saturating_sub(stat.start_time / ticks_per_sec as u64))
        }
    } else {
        Duration::ZERO
    };

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
            uid,
            user,
            time,
            #[cfg(feature = "gpu")]
            gpu_mem: 0,
            #[cfg(feature = "gpu")]
            gpu_mem_percent: 0.0,
            #[cfg(feature = "gpu")]
            gpu_util: 0,
        },
        new_process_times,
    ))
}

pub(crate) struct PrevProc {
    pub prev_idle: f64,
    pub prev_non_idle: f64,
}

#[derive(Clone, Copy)]
pub(crate) struct ProcHarvestOptions {
    pub use_current_cpu_total: bool,
    pub unnormalized_cpu: bool,
}

fn is_str_numeric(s: &str) -> bool {
    s.chars().all(|c| c.is_ascii_digit())
}

/// General args to keep around for reading proc data.
#[derive(Copy, Clone)]
pub(crate) struct ReadProcArgs {
    pub(crate) use_current_cpu_total: bool,
    pub(crate) cpu_usage: f64,
    pub(crate) cpu_fraction: f64,
    pub(crate) total_memory: u64,
    pub(crate) time_difference_in_secs: u64,
    pub(crate) uptime: u64,
}

pub struct ProcessCollector {
    pub options: ProcHarvestOptions,
    pub prev_proc: PrevProc,
    pub pid_mapping: HashMap<Pid, PrevProcDetails>,
    pub user_table: UserTable,

    #[cfg(feature = "gpu")]
    pub gpu_pids: Option<Vec<HashMap<u32, (u64, u32)>>>,
}

pub(crate) fn linux_process_data(
    system: &sysinfo::System, time_difference_in_secs: u64, collector: &mut ProcessCollector,
    #[cfg(feature = "gpu")] gpus_total_mem: Option<u64>,
) -> CollectionResult<Vec<ProcessHarvest>> {
    let total_memory = system.total_memory();

    let ProcHarvestOptions {
        use_current_cpu_total,
        unnormalized_cpu,
    } = collector.options;

    let PrevProc {
        prev_idle,
        prev_non_idle,
    } = &mut collector.prev_proc;

    // TODO: [PROC THREADS] Add threads

    let CpuUsage {
        mut cpu_usage,
        cpu_fraction,
    } = cpu_usage_calculation(prev_idle, prev_non_idle)?;

    if unnormalized_cpu {
        let num_processors = system.cpus().len() as f64;

        // Note we *divide* here because the later calculation divides `cpu_usage` - in
        // effect, multiplying over the number of cores.
        cpu_usage /= num_processors;
    }

    let mut pids_to_clear: HashSet<Pid> = collector.pid_mapping.keys().cloned().collect();

    let pids = fs::read_dir("/proc")?.flatten().filter_map(|dir| {
        if is_str_numeric(dir.file_name().to_string_lossy().trim()) {
            Some(dir.path())
        } else {
            None
        }
    });

    let args = ReadProcArgs {
        use_current_cpu_total,
        cpu_usage,
        cpu_fraction,
        total_memory,
        time_difference_in_secs,
        uptime: sysinfo::System::uptime(),
    };

    let process_vector: Vec<ProcessHarvest> = pids
        .filter_map(|pid_path| {
            if let Ok(process) = Process::from_path(pid_path) {
                let pid = process.pid;
                let prev_proc_details = collector.pid_mapping.entry(pid).or_default();

                #[allow(unused_mut)]
                if let Ok((mut process_harvest, new_process_times)) =
                    read_proc(prev_proc_details, process, args, &mut collector.user_table)
                {
                    #[cfg(feature = "gpu")]
                    if let Some(gpus) = &collector.gpu_pids {
                        gpus.iter().for_each(|gpu| {
                            // add mem/util for all gpus to pid
                            if let Some((mem, util)) = gpu.get(&(pid as u32)) {
                                process_harvest.gpu_mem += mem;
                                process_harvest.gpu_util += util;
                            }
                        });

                        if let Some(gpu_total_mem) = gpus_total_mem {
                            process_harvest.gpu_mem_percent =
                                (process_harvest.gpu_mem as f64 / gpu_total_mem as f64 * 100.0)
                                    as f32;
                        }
                    }

                    prev_proc_details.cpu_time = new_process_times;
                    prev_proc_details.total_read_bytes = process_harvest.total_read_bytes;
                    prev_proc_details.total_write_bytes = process_harvest.total_write_bytes;

                    pids_to_clear.remove(&pid);
                    return Some(process_harvest);
                }
            }

            None
        })
        .collect();

    pids_to_clear.iter().for_each(|pid| {
        collector.pid_mapping.remove(pid);
    });

    Ok(process_vector)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_proc_cpu_parse() {
        assert_eq!(
            (100_f64, 200_f64),
            fetch_cpu_usage("100 0 100 100"),
            "Failed to properly calculate idle/non-idle for /proc/stat CPU with 4 values"
        );
        assert_eq!(
            (120_f64, 200_f64),
            fetch_cpu_usage("100 0 100 100 20"),
            "Failed to properly calculate idle/non-idle for /proc/stat CPU with 5 values"
        );
        assert_eq!(
            (120_f64, 230_f64),
            fetch_cpu_usage("100 0 100 100 20 30"),
            "Failed to properly calculate idle/non-idle for /proc/stat CPU with 6 values"
        );
        assert_eq!(
            (120_f64, 270_f64),
            fetch_cpu_usage("100 0 100 100 20 30 40"),
            "Failed to properly calculate idle/non-idle for /proc/stat CPU with 7 values"
        );
        assert_eq!(
            (120_f64, 320_f64),
            fetch_cpu_usage("100 0 100 100 20 30 40 50"),
            "Failed to properly calculate idle/non-idle for /proc/stat CPU with 8 values"
        );
        assert_eq!(
            (120_f64, 320_f64),
            fetch_cpu_usage("100 0 100 100 20 30 40 50 100"),
            "Failed to properly calculate idle/non-idle for /proc/stat CPU with 9 values"
        );
        assert_eq!(
            (120_f64, 320_f64),
            fetch_cpu_usage("100 0 100 100 20 30 40 50 100 200"),
            "Failed to properly calculate idle/non-idle for /proc/stat CPU with 10 values"
        );
    }
}
