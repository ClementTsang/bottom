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
        #[cfg(target_os = "freebsd")]
        {
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
        }
    };
    Some(MemHarvest {
        mem_total_in_kib,
        mem_used_in_kib,
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
