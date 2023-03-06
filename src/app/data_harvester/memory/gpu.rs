use super::MemHarvest;

/// Return GPU data. Currently only supports NVIDIA cards.
#[cfg(feature = "nvidia")]
pub(crate) fn get_gpu_data() -> Option<Vec<(String, MemHarvest)>> {
    use crate::data_harvester::nvidia::NVML_DATA;

    if let Ok(nvml) = &*NVML_DATA {
        if let Ok(num_gpu) = nvml.device_count() {
            let mut results = Vec::with_capacity(num_gpu as usize);
            for i in 0..num_gpu {
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
