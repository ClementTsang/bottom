//! Collecting memory data using sysinfo.

use sysinfo::{System, SystemExt};

use crate::data_harvester::memory::{MemCollect, MemHarvest};

/// Returns all memory data.
pub(crate) fn get_mem_data(sys: &System, _get_gpu: bool) -> MemCollect {
    MemCollect {
        ram: get_ram_data(sys),
        swap: get_swap_data(sys),
        #[cfg(feature = "zfs")]
        arc: get_arc_data(),
        #[cfg(feature = "gpu")]
        gpus: if _get_gpu { get_gpu_data() } else { None },
    }
}

/// Returns RAM usage.
pub(crate) fn get_ram_data(sys: &System) -> Option<MemHarvest> {
    let mem_used_in_kib = sys.used_memory() / 1024;
    let mem_total_in_kib = sys.total_memory() / 1024;

    Some(MemHarvest {
        total_kib: mem_total_in_kib,
        used_kib: mem_used_in_kib,
        use_percent: if mem_total_in_kib == 0 {
            None
        } else {
            Some(mem_used_in_kib as f64 / mem_total_in_kib as f64 * 100.0)
        },
    })
}

/// Returns SWAP usage.
pub(crate) fn get_swap_data(sys: &System) -> Option<MemHarvest> {
    let mem_used_in_kib = sys.used_swap() / 1024;
    let mem_total_in_kib = sys.total_swap() / 1024;

    Some(MemHarvest {
        total_kib: mem_total_in_kib,
        used_kib: mem_used_in_kib,
        use_percent: if mem_total_in_kib == 0 {
            None
        } else {
            Some(mem_used_in_kib as f64 / mem_total_in_kib as f64 * 100.0)
        },
    })
}

/// Return ARC usage.
#[cfg(feature = "zfs")]
pub(crate) fn get_arc_data() -> Option<MemHarvest> {
    let (mem_total_in_kib, mem_used_in_kib) = {
        cfg_if::cfg_if! {
            if #[cfg(target_os = "linux")]
            {
                // TODO: [OPT] is this efficient?
                use std::fs::read_to_string;
                if let Ok(arc_stats) = read_to_string("/proc/spl/kstat/zfs/arcstats") {
                    let mut mem_arc = 0;
                    let mut mem_total = 0;
                    let mut zfs_keys_read: u8 = 0;
                    const ZFS_KEYS_NEEDED: u8 = 2;

                    for line in arc_stats.lines() {
                        if let Some((label, value)) = line.split_once(' ') {
                            let to_write = match label {
                                "size" => &mut mem_arc,
                                "memory_all_bytes" => &mut mem_total,
                                _ => {
                                    continue;
                                }
                            };

                            if let Some((_type, number)) = value.trim_start().rsplit_once(' ') {
                                // Parse the value, remember it's in bytes!
                                if let Ok(number) = number.parse::<u64>() {
                                    *to_write = number;
                                    // We only need a few keys, so we can bail early.
                                    zfs_keys_read += 1;
                                    if zfs_keys_read == ZFS_KEYS_NEEDED {
                                        break;
                                    }
                                }
                            }
                        }
                    }
                    (mem_total / 1024, mem_arc / 1024)
                } else {
                    (0, 0)
                }
            } else if #[cfg(target_os = "freebsd")] {
                use sysctl::Sysctl;
                if let (Ok(mem_arc_value), Ok(mem_sys_value)) = (
                    sysctl::Ctl::new("kstat.zfs.misc.arcstats.size"),
                    sysctl::Ctl::new("hw.physmem"),
                ) {
                    if let (Ok(sysctl::CtlValue::U64(arc)), Ok(sysctl::CtlValue::Ulong(mem))) =
                        (mem_arc_value.value(), mem_sys_value.value())
                    {
                        (mem / 1024, arc / 1024)
                    } else {
                        (0, 0)
                    }
                } else {
                    (0, 0)
                }
            } else {
                (0, 0)
            }
        }
    };

    Some(MemHarvest {
        total_kib: mem_total_in_kib,
        used_kib: mem_used_in_kib,
        use_percent: if mem_total_in_kib == 0 {
            None
        } else {
            Some(mem_used_in_kib as f64 / mem_total_in_kib as f64 * 100.0)
        },
    })
}

/// Return GPU data. Currently only supports NVIDIA cards.
#[cfg(feature = "nvidia")]
pub(crate) fn get_gpu_data() -> Option<Vec<(String, MemHarvest)>> {
    use crate::data_harvester::nvidia::NVML_DATA;

    if let Ok(nvml) = &*NVML_DATA {
        if let Ok(ngpu) = nvml.device_count() {
            let mut results = Vec::with_capacity(ngpu as usize);
            for i in 0..ngpu {
                if let Ok(device) = nvml.device_by_index(i) {
                    if let (Ok(name), Ok(mem)) = (device.name(), device.memory_info()) {
                        // add device memory in bytes
                        let mem_total_in_kib = mem.total / 1024;
                        let mem_used_in_kib = mem.used / 1024;
                        results.push((
                            name,
                            MemHarvest {
                                total_kib: mem_total_in_kib,
                                used_kib: mem_used_in_kib,
                                use_percent: if mem_total_in_kib == 0 {
                                    None
                                } else {
                                    Some(mem_used_in_kib as f64 / mem_total_in_kib as f64 * 100.0)
                                },
                            },
                        ));
                    }
                }
            }
            Some(results)
        } else {
            None
        }
    } else {
        None
    }
}
