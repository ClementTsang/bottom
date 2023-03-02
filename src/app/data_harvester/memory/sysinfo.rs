//! Data collection for memory via sysinfo.

use sysinfo::{System, SystemExt};

use crate::data_harvester::memory::{MemCollect, MemHarvest};

pub async fn get_mem_data(sys: &System, actually_get: bool, _get_gpu: bool) -> MemCollect {
    if !actually_get {
        MemCollect {
            ram: Ok(None),
            swap: Ok(None),
            #[cfg(feature = "zfs")]
            arc: Ok(None),
            #[cfg(feature = "gpu")]
            gpus: Ok(None),
        }
    } else {
        MemCollect {
            ram: get_ram_data(sys).await,
            swap: get_swap_data(sys).await,
            #[cfg(feature = "zfs")]
            arc: get_arc_data().await,
            #[cfg(feature = "gpu")]
            gpus: if _get_gpu {
                get_gpu_data().await
            } else {
                Ok(None)
            },
        }
    }
}

pub async fn get_ram_data(sys: &System) -> crate::utils::error::Result<Option<MemHarvest>> {
    let (mem_total_in_kib, mem_used_in_kib) = (sys.total_memory() / 1024, sys.used_memory() / 1024);

    Ok(Some(MemHarvest {
        mem_total_in_kib,
        mem_used_in_kib,
        use_percent: if mem_total_in_kib == 0 {
            None
        } else {
            Some(mem_used_in_kib as f64 / mem_total_in_kib as f64 * 100.0)
        },
    }))
}

pub async fn get_swap_data(sys: &System) -> crate::utils::error::Result<Option<MemHarvest>> {
    let (mem_total_in_kib, mem_used_in_kib) = (sys.total_swap() / 1024, sys.used_swap() / 1024);

    Ok(Some(MemHarvest {
        mem_total_in_kib,
        mem_used_in_kib,
        use_percent: if mem_total_in_kib == 0 {
            None
        } else {
            Some(mem_used_in_kib as f64 / mem_total_in_kib as f64 * 100.0)
        },
    }))
}

#[cfg(feature = "zfs")]
pub async fn get_arc_data() -> crate::utils::error::Result<Option<MemHarvest>> {
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
    Ok(Some(MemHarvest {
        mem_total_in_kib,
        mem_used_in_kib,
        use_percent: if mem_total_in_kib == 0 {
            None
        } else {
            Some(mem_used_in_kib as f64 / mem_total_in_kib as f64 * 100.0)
        },
    }))
}

#[cfg(feature = "nvidia")]
pub async fn get_gpu_data() -> crate::utils::error::Result<Option<Vec<(String, MemHarvest)>>> {
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
                                mem_total_in_kib,
                                mem_used_in_kib,
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
            Ok(Some(results))
        } else {
            Ok(None)
        }
    } else {
        Ok(None)
    }
}
