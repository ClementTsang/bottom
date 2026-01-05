//! Process data collection for Windows. Uses sysinfo.

use std::time::Duration;

use anyhow::bail;
use itertools::Itertools;
use windows::Win32::{
    Foundation::{CloseHandle, HANDLE},
    System::Threading::{GetPriorityClass, OpenProcess, PROCESS_QUERY_LIMITED_INFORMATION},
};

use super::{ProcessHarvest, process_status_str};
use crate::collection::{DataCollector, error::CollectionResult};

/// See [here](https://learn.microsoft.com/en-us/windows/win32/api/processthreadsapi/nf-processthreadsapi-getpriorityclass)
/// for more information on the core Windows API being called and the meaning of the priorities, as well as the access
/// rights needed.
fn get_priority(pid: u32) -> anyhow::Result<i32> {
    // SAFETY: We check validity of each step and bail on errors. We also close the handle.
    unsafe {
        let process_handle: HANDLE = OpenProcess(PROCESS_QUERY_LIMITED_INFORMATION, false, pid)?;
        if process_handle.is_invalid() {
            bail!("Failed to open process with PID {pid} to get priority class");
        }

        // From docs: "If the function fails, the return value is zero."
        let priority = GetPriorityClass(process_handle);
        if priority == 0 {
            bail!("Failed to get priority class for process with PID {pid}");
        }

        let handle_result = CloseHandle(process_handle);
        if let Err(err) = handle_result {
            bail!(err);
        }

        Ok(priority as i32)
    }
}

// TODO: There's a lot of shared code with this and the unix impl.
pub fn sysinfo_process_data(
    collector: &mut DataCollector,
) -> CollectionResult<Vec<ProcessHarvest>> {
    let sys = &collector.sys.system;
    let users = &collector.sys.users;
    let use_current_cpu_total = collector.use_current_cpu_total;
    let unnormalized_cpu = collector.unnormalized_cpu;
    let total_memory = collector.total_memory();

    let mut process_vector: Vec<ProcessHarvest> = Vec::new();
    let process_hashmap = sys.processes();
    let cpu_usage = sys.global_cpu_usage() / 100.0;
    let num_processors = sys.cpus().len();

    for process in process_hashmap.values() {
        let name = if process.name().is_empty() {
            let process_cmd = process.cmd();
            if process_cmd.len() > 1 {
                process_cmd[0].to_string_lossy().to_string()
            } else {
                process
                    .exe()
                    .and_then(|exe| exe.file_stem())
                    .and_then(|stem| stem.to_str())
                    .map(|s| s.to_string())
                    .unwrap_or(String::new())
            }
        } else {
            process.name().to_string_lossy().to_string()
        };
        let command = {
            let command = process.cmd().iter().map(|s| s.to_string_lossy()).join(" ");
            if command.is_empty() {
                name.clone()
            } else {
                command
            }
        };

        let process_cpu_usage = {
            let pcu = {
                let usage = process.cpu_usage();
                if unnormalized_cpu || num_processors == 0 {
                    usage
                } else {
                    usage / num_processors as f32
                }
            };

            if use_current_cpu_total && cpu_usage > 0.0 {
                pcu / cpu_usage
            } else {
                pcu
            }
        };

        let disk_usage = process.disk_usage();
        let process_state = (process_status_str(process.status()), 'R');

        #[cfg(feature = "gpu")]
        let (gpu_mem, gpu_util, gpu_mem_percent) = {
            let mut gpu_mem = 0;
            let mut gpu_util = 0;
            let mut gpu_mem_percent = 0.0;
            if let Some(gpus) = &collector.gpu_pids {
                use crate::collection::processes::Pid;

                gpus.iter().for_each(|gpu| {
                    // add mem/util for all gpus to pid
                    if let Some((mem, util)) = gpu.get(&(process.pid().as_u32() as Pid)) {
                        gpu_mem += mem;
                        gpu_util += util;
                    }
                });
            }
            if let Some(gpu_total_mem) = &collector.gpus_total_mem {
                gpu_mem_percent = (gpu_mem as f64 / *gpu_total_mem as f64 * 100.0) as f32;
            }
            (gpu_mem, gpu_util, gpu_mem_percent)
        };

        let pid = process.pid().as_u32();
        let priority = get_priority(pid).unwrap_or(0);

        process_vector.push(ProcessHarvest {
            pid: pid as _,
            parent_pid: process.parent().map(|p| p.as_u32() as _),
            name,
            command,
            mem_usage_percent: if total_memory > 0 {
                process.memory() as f64 * 100.0 / total_memory as f64
            } else {
                0.0
            } as f32,
            mem_usage: process.memory(),
            virtual_mem: process.virtual_memory(),
            cpu_usage_percent: process_cpu_usage,
            read_per_sec: disk_usage.read_bytes,
            write_per_sec: disk_usage.written_bytes,
            total_read: disk_usage.total_read_bytes,
            total_write: disk_usage.total_written_bytes,
            process_state,
            user: process
                .user_id()
                .and_then(|uid| users.get_user_by_id(uid).map(|user| user.name().into())),
            time: if process.start_time() == 0 {
                // Workaround for sysinfo occasionally returning a start time equal to UNIX
                // epoch, giving a run time in the range of 50+ years. We just
                // return a time of zero in this case for simplicity.
                //
                // TODO: Maybe return an option instead?
                Duration::ZERO
            } else {
                Duration::from_secs(process.run_time())
            },
            #[cfg(feature = "gpu")]
            gpu_mem,
            #[cfg(feature = "gpu")]
            gpu_util,
            #[cfg(feature = "gpu")]
            gpu_mem_percent,
            priority, // TODO: Translate this to Windows priority names?
        });
    }

    Ok(process_vector)
}
