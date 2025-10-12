//! Shared process data harvesting code from macOS and FreeBSD via sysinfo.

use std::{io, time::Duration};

use hashbrown::HashMap;
use itertools::Itertools;
use sysinfo::{ProcessStatus, System};

use super::{ProcessHarvest, process_status_str};
use crate::collection::{Pid, error::CollectionResult, processes::UserTable};

pub(crate) trait UnixProcessExt {
    fn sysinfo_process_data(
        sys: &System, use_current_cpu_total: bool, unnormalized_cpu: bool, total_memory: u64,
        user_table: &mut UserTable,
    ) -> CollectionResult<Vec<ProcessHarvest>> {
        let mut process_vector: Vec<ProcessHarvest> = Vec::new();
        let process_hashmap = sys.processes();
        let cpu_usage = sys.global_cpu_usage() / 100.0;
        let num_processors = sys.cpus().len();

        for process_val in process_hashmap.values() {
            let name = if process_val.name().is_empty() {
                let process_cmd = process_val.cmd();
                if let Some(name) = process_cmd.first() {
                    name.to_string_lossy().to_string()
                } else {
                    process_val
                        .exe()
                        .and_then(|exe| exe.file_stem())
                        .and_then(|stem| stem.to_str())
                        .map(|s| s.to_string())
                        .unwrap_or(String::new())
                }
            } else {
                process_val.name().to_string_lossy().to_string()
            };
            let command = {
                let command = process_val
                    .cmd()
                    .iter()
                    .map(|s| s.to_string_lossy())
                    .join(" ");
                if command.is_empty() {
                    name.clone()
                } else {
                    command
                }
            };

            let pcu = {
                let usage = process_val.cpu_usage();
                if unnormalized_cpu || num_processors == 0 {
                    usage
                } else {
                    usage / num_processors as f32
                }
            };
            let process_cpu_usage = if use_current_cpu_total && cpu_usage > 0.0 {
                pcu / cpu_usage
            } else {
                pcu
            };

            let disk_usage = process_val.disk_usage();
            let process_state = {
                let ps = process_val.status();
                (process_status_str(ps), convert_process_status_to_char(ps))
            };
            let uid = process_val.user_id().map(|u| **u);
            let pid = process_val.pid().as_u32() as Pid;
            process_vector.push(ProcessHarvest {
                pid,
                parent_pid: Self::parent_pid(process_val),
                name,
                command,
                mem_usage_percent: if total_memory > 0 {
                    (process_val.memory() as f64 * 100.0 / total_memory as f64) as f32
                } else {
                    0.0
                },
                mem_usage: process_val.memory(),
                virtual_mem: process_val.virtual_memory(),
                cpu_usage_percent: process_cpu_usage,
                read_per_sec: disk_usage.read_bytes,
                write_per_sec: disk_usage.written_bytes,
                total_read: disk_usage.total_read_bytes,
                total_write: disk_usage.total_written_bytes,
                process_state,
                uid,
                user: uid.and_then(|uid| user_table.uid_to_username(uid).ok()),
                time: if process_val.start_time() == 0 {
                    // Workaround for sysinfo occasionally returning a start time equal to UNIX
                    // epoch, giving a run time in the range of 50+ years. We just
                    // return a time of zero in this case for simplicity.
                    //
                    // TODO: Maybe return an option instead?
                    Duration::ZERO
                } else {
                    Duration::from_secs(process_val.run_time())
                },
                #[cfg(feature = "gpu")]
                gpu_mem: 0,
                #[cfg(feature = "gpu")]
                gpu_mem_percent: 0.0,
                #[cfg(feature = "gpu")]
                gpu_util: 0,
            });
        }

        if Self::has_backup_proc_cpu_fn() {
            let unknown_state = ProcessStatus::Unknown(0).to_string();
            let cpu_usage_unknown_pids: Vec<Pid> = process_vector
                .iter()
                .filter(|process| process.process_state.0 == unknown_state)
                .map(|process| process.pid)
                .collect();
            let cpu_usages = Self::backup_proc_cpu(&cpu_usage_unknown_pids)?;
            for process in &mut process_vector {
                if cpu_usages.contains_key(&process.pid) {
                    process.cpu_usage_percent = if unnormalized_cpu || num_processors == 0 {
                        *cpu_usages.get(&process.pid).unwrap()
                    } else {
                        *cpu_usages.get(&process.pid).unwrap() / num_processors as f32
                    };
                }
            }
        }

        Ok(process_vector)
    }

    #[inline]
    fn has_backup_proc_cpu_fn() -> bool {
        false
    }

    fn backup_proc_cpu(_pids: &[Pid]) -> io::Result<HashMap<Pid, f32>> {
        Ok(HashMap::default())
    }

    fn parent_pid(process_val: &sysinfo::Process) -> Option<Pid> {
        process_val.parent().map(|p| p.as_u32() as _)
    }
}

fn convert_process_status_to_char(status: ProcessStatus) -> char {
    // TODO: Based on https://github.com/GuillaumeGomez/sysinfo/blob/baa46efb46d82f21b773088603720262f4a34646/src/unix/freebsd/process.rs#L13?
    cfg_if::cfg_if! {
        if #[cfg(target_os = "macos")] {
            // SAFETY: These are all const and should be valid characters.
            const SIDL: char = unsafe { char::from_u32_unchecked(libc::SIDL) };

            // SAFETY: These are all const and should be valid characters.
            const SRUN: char = unsafe { char::from_u32_unchecked(libc::SRUN) };

            // SAFETY: These are all const and should be valid characters.
            const SSLEEP: char = unsafe { char::from_u32_unchecked(libc::SSLEEP) };

            // SAFETY: These are all const and should be valid characters.
            const SSTOP: char = unsafe { char::from_u32_unchecked(libc::SSTOP) };

            // SAFETY: These are all const and should be valid characters.
            const SZOMB: char = unsafe { char::from_u32_unchecked(libc::SZOMB) };

            match status {
                ProcessStatus::Idle => SIDL,
                ProcessStatus::Run => SRUN,
                ProcessStatus::Sleep => SSLEEP,
                ProcessStatus::Stop => SSTOP,
                ProcessStatus::Zombie => SZOMB,
                _ => '?'
            }
        } else if #[cfg(target_os = "freebsd")] {
            const fn assert_u8(val: libc::c_char) -> u8 {
                if val < 0 { panic!("there was an invalid i8 constant that is supposed to be a char") } else { val as u8 }
            }

            const SIDL: u8 = assert_u8(libc::SIDL);
            const SRUN: u8 = assert_u8(libc::SRUN);
            const SSLEEP: u8 = assert_u8(libc::SSLEEP);
            const SSTOP: u8 = assert_u8(libc::SSTOP);
            const SZOMB: u8 = assert_u8(libc::SZOMB);
            const SWAIT: u8 = assert_u8(libc::SWAIT);
            const SLOCK: u8 = assert_u8(libc::SLOCK);

            match status {
                ProcessStatus::Idle => SIDL as char,
                ProcessStatus::Run => SRUN as char,
                ProcessStatus::Sleep => SSLEEP as char,
                ProcessStatus::Stop => SSTOP as char,
                ProcessStatus::Zombie => SZOMB as char,
                ProcessStatus::Dead => SWAIT as char,
                ProcessStatus::LockBlocked => SLOCK as char,
                _ => '?'
            }
        } else {
            '?'
        }
    }
}
