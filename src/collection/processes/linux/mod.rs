//! Process data collection for Linux.

mod process;

use std::{
    fs::{self, File},
    io::{BufRead, BufReader},
    time::Duration,
};

use concat_string::concat_string;
use hashbrown::{HashMap, HashSet};
use process::*;
use sysinfo::ProcessStatus;

use super::{Pid, ProcessHarvest, UserTable, process_status_str};
use crate::collection::{DataCollector, error::CollectionResult, processes::ProcessType};

/// Maximum character length of a `/proc/<PID>/stat` process name (the length is 16,
/// but this includes a null terminator).
///
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
    thread_parent: Option<Pid>,
) -> CollectionResult<(ProcessHarvest, u64)> {
    let Process {
        pid: _pid,
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
        system_uptime,
        get_process_threads: _,
    } = args;

    let process_state_char = stat.state;
    let process_state = (
        process_status_str(ProcessStatus::from(process_state_char)),
        process_state_char,
    );
    let (cpu_usage_percent, new_process_times) = get_linux_cpu_usage(
        &stat,
        cpu_usage,
        cpu_fraction,
        prev_proc.cpu_time,
        use_current_cpu_total,
    );

    let (parent_pid, process_type) = if let Some(thread_parent) = thread_parent {
        (Some(thread_parent), ProcessType::ProcessThread)
    } else if stat.is_kernel_thread {
        (Some(stat.ppid), ProcessType::Kernel)
    } else {
        (Some(stat.ppid), ProcessType::Regular)
    };

    let mem_usage = stat.rss_bytes();
    let mem_usage_percent = (mem_usage as f64 / total_memory as f64 * 100.0) as f32;
    let virtual_mem = stat.vsize;

    // XXX: This can fail if permission is denied.
    let (total_read, total_write, read_per_sec, write_per_sec) = if let Some(io) = io {
        let total_read = io.read_bytes;
        let total_write = io.write_bytes;
        let prev_total_read = prev_proc.total_read_bytes;
        let prev_total_write = prev_proc.total_write_bytes;

        let read_per_sec = total_read
            .saturating_sub(prev_total_read)
            .checked_div(time_difference_in_secs)
            .unwrap_or(0);

        let write_per_sec = total_write
            .saturating_sub(prev_total_write)
            .checked_div(time_difference_in_secs)
            .unwrap_or(0);

        (total_read, total_write, read_per_sec, write_per_sec)
    } else {
        (0, 0, 0, 0)
    };

    let user = uid.and_then(|uid| user_table.uid_to_username(uid).ok());

    let time = if let Ok(ticks_per_sec) = u32::try_from(rustix::param::clock_ticks_per_second()) {
        if ticks_per_sec == 0 {
            Duration::ZERO
        } else {
            Duration::from_secs(
                system_uptime.saturating_sub(stat.start_time / ticks_per_sec as u64),
            )
        }
    } else {
        Duration::ZERO
    };

    let (command, name) = {
        let comm = stat.comm;
        if let Some(cmdline) = cmdline {
            if cmdline.is_empty() {
                (concat_string!("[", comm, "]"), comm)
            } else {
                // If the comm fits then we'll default to whatever is set.
                // If it doesn't, we need to do some magic to determine what it's
                // supposed to be.

                // TODO: We might want to re-evaluate if we want to do it like this,
                // as it turns out I was dumb and sometimes comm != process name...
                //
                // What we should do is store:
                // - basename (what we're kinda doing now, except we're gating on comm length)
                // - command (full thing)
                // - comm (as a separate thing)
                //
                // Stuff like htop also offers the option to "highlight" basename and comm in command. Might be neat?
                let name = if comm.len() >= MAX_STAT_NAME_LEN {
                    binary_name_from_cmdline(&cmdline)
                } else {
                    comm
                };

                (cmdline, name)
            }
        } else {
            (comm.clone(), comm)
        }
    };

    // We have moved command processing here.
    // SAFETY: We are only replacing a single char (NUL) with another single char (space).

    let mut command = command;
    let buf_mut = unsafe { command.as_mut_vec() };

    for byte in buf_mut {
        if *byte == 0 {
            const SPACE: u8 = ' '.to_ascii_lowercase() as u8;
            *byte = SPACE;
        }
    }

    Ok((
        ProcessHarvest {
            pid: process.pid,
            parent_pid,
            cpu_usage_percent,
            mem_usage_percent,
            mem_usage,
            virtual_mem,
            name,
            command,
            read_per_sec,
            write_per_sec,
            total_read,
            total_write,
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
            process_type,
        },
        new_process_times,
    ))
}

/// We follow something similar to how htop does it to identify a valid name based on the cmdline.
/// - https://github.com/htop-dev/htop/blob/bcb18ef82269c68d54a160290e5f8b2e939674ec/Process.c#L268 (kinda)
/// - https://github.com/htop-dev/htop/blob/bcb18ef82269c68d54a160290e5f8b2e939674ec/Process.c#L573
///
/// Also note that cmdline is (for us) separated by \0.
fn binary_name_from_cmdline(cmdline: &str) -> String {
    let mut start = 0;
    let mut end = cmdline.len();

    for (i, c) in cmdline.chars().enumerate() {
        if c == '/' {
            start = i + 1;
        } else if c == '\0' || c == ':' {
            end = i;
            break;
        }
    }

    // Bit of a hack to handle cases like "firefox -blah"
    let partial = &cmdline[start..end];
    partial
        .split_once(" -")
        .map(|(name, _)| name.to_string())
        .unwrap_or_else(|| partial.to_string())
}

pub(crate) struct PrevProc<'a> {
    pub prev_idle: &'a mut f64,
    pub prev_non_idle: &'a mut f64,
}

pub(crate) struct ProcHarvestOptions {
    pub use_current_cpu_total: bool,
    pub unnormalized_cpu: bool,
    pub get_process_threads: bool,
}

fn is_str_numeric(s: &str) -> bool {
    s.chars().all(|c| c.is_ascii_digit())
}

/// General args to keep around for reading proc data.
#[derive(Copy, Clone)]
pub(crate) struct ReadProcArgs {
    pub use_current_cpu_total: bool,
    pub cpu_usage: f64,
    pub cpu_fraction: f64,
    pub total_memory: u64,
    pub time_difference_in_secs: u64,
    pub system_uptime: u64,
    pub get_process_threads: bool,
}

pub(crate) fn linux_process_data(
    collector: &mut DataCollector, time_difference_in_secs: u64,
) -> CollectionResult<Vec<ProcessHarvest>> {
    let total_memory = collector.total_memory();
    let prev_proc = PrevProc {
        prev_idle: &mut collector.prev_idle,
        prev_non_idle: &mut collector.prev_non_idle,
    };
    let proc_harvest_options = ProcHarvestOptions {
        use_current_cpu_total: collector.use_current_cpu_total,
        unnormalized_cpu: collector.unnormalized_cpu,
        get_process_threads: collector.get_process_threads,
    };
    let prev_process_details = &mut collector.prev_process_details;
    let user_table = &mut collector.user_table;

    let ProcHarvestOptions {
        use_current_cpu_total,
        unnormalized_cpu,
        get_process_threads: get_threads,
    } = proc_harvest_options;

    let PrevProc {
        prev_idle,
        prev_non_idle,
    } = prev_proc;

    // TODO: [PROC THREADS] Add threads

    let CpuUsage {
        mut cpu_usage,
        cpu_fraction,
    } = cpu_usage_calculation(prev_idle, prev_non_idle)?;

    if unnormalized_cpu {
        let num_processors = collector.sys.system.cpus().len() as f64;

        // Note we *divide* here because the later calculation divides `cpu_usage` - in
        // effect, multiplying over the number of cores.
        cpu_usage /= num_processors;
    }

    // TODO: Could maybe use a double buffer hashmap to avoid allocating this each time?
    // e.g. we swap which is prev and which is new.
    let mut seen_pids: HashSet<Pid> = HashSet::new();

    // Note this will only return PIDs of _processes_, not threads. You can get those from /proc/<PID>/task though.
    let pids = fs::read_dir("/proc")?.flatten().filter_map(|dir| {
        // Need to filter out non-PID entries.
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
        system_uptime: sysinfo::System::uptime(),
        get_process_threads: get_threads,
    };

    // TODO: Maybe pre-allocate these buffers in the future w/ routine cleanup.
    let mut buffer = String::new();
    let mut process_threads_to_check = HashMap::new();

    let mut process_vector: Vec<ProcessHarvest> = pids
        .filter_map(|pid_path| {
            if let Ok((process, threads)) =
                Process::from_path(pid_path, &mut buffer, args.get_process_threads)
            {
                let pid = process.pid;
                let prev_proc_details = prev_process_details.entry(pid).or_default();

                #[cfg_attr(not(feature = "gpu"), expect(unused_mut))]
                if let Ok((mut process_harvest, new_process_times)) =
                    read_proc(prev_proc_details, process, args, user_table, None)
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
                        if let Some(gpu_total_mem) = &collector.gpus_total_mem {
                            process_harvest.gpu_mem_percent =
                                (process_harvest.gpu_mem as f64 / *gpu_total_mem as f64 * 100.0)
                                    as f32;
                        }
                    }

                    prev_proc_details.cpu_time = new_process_times;
                    prev_proc_details.total_read_bytes = process_harvest.total_read;
                    prev_proc_details.total_write_bytes = process_harvest.total_write;

                    if !threads.is_empty() {
                        process_threads_to_check.insert(pid, threads);
                    }

                    seen_pids.insert(pid);
                    return Some(process_harvest);
                }
            }

            None
        })
        .collect();

    // Get thread data.
    for (pid, tid_paths) in process_threads_to_check {
        for tid_path in tid_paths {
            if let Ok((process, _)) = Process::from_path(tid_path, &mut buffer, false) {
                let tid = process.pid;
                let prev_proc_details = prev_process_details.entry(tid).or_default();

                if let Ok((process_harvest, new_process_times)) =
                    read_proc(prev_proc_details, process, args, user_table, Some(pid))
                {
                    prev_proc_details.cpu_time = new_process_times;
                    prev_proc_details.total_read_bytes = process_harvest.total_read;
                    prev_proc_details.total_write_bytes = process_harvest.total_write;

                    seen_pids.insert(tid);
                    process_vector.push(process_harvest);
                }
            }
        }
    }

    // Clean up values we don't care about anymore.
    prev_process_details.retain(|pid, _| seen_pids.contains(pid));

    // Occasional garbage collection.
    if collector.should_run_less_routine_tasks {
        prev_process_details.shrink_to_fit();
    }

    // TODO: This might be more efficient to just separate threads into their own list, but for now this works so it
    // fits with existing code.
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

    #[test]
    fn test_name_from_cmdline() {
        assert_eq!(binary_name_from_cmdline("/usr/bin/btm"), "btm");
        assert_eq!(
            binary_name_from_cmdline("/usr/bin/btm\0--asdf\0--asdf/gkj"),
            "btm"
        );
        assert_eq!(binary_name_from_cmdline("/usr/bin/btm:"), "btm");
        assert_eq!(binary_name_from_cmdline("/usr/bin/b tm"), "b tm");
        assert_eq!(binary_name_from_cmdline("/usr/bin/b tm:"), "b tm");
        assert_eq!(binary_name_from_cmdline("/usr/bin/b tm\0--test"), "b tm");
        assert_eq!(binary_name_from_cmdline("/usr/bin/b tm:\0--test"), "b tm");
        assert_eq!(
            binary_name_from_cmdline("/usr/bin/b t m:\0--\"test thing\""),
            "b t m"
        );
        assert_eq!(
            binary_name_from_cmdline("firefox -contentproc -isForBrowser -prefsHandle 0"),
            "firefox"
        );
    }
}
