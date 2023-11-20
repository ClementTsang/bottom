use hashbrown::HashMap;
use nvml_wrapper::enum_wrappers::device::TemperatureSensor;
use nvml_wrapper::enums::device::UsedGpuMemory;
use nvml_wrapper::{error::NvmlError, Nvml};
use once_cell::sync::Lazy;

use crate::app::Filter;

use crate::app::layout_manager::UsedWidgets;
use crate::data_harvester::memory::MemHarvest;
use crate::data_harvester::temperature::{is_temp_filtered, TempHarvest, TemperatureType};

pub static NVML_DATA: Lazy<Result<Nvml, NvmlError>> = Lazy::new(Nvml::init);

pub struct GpusData {
    pub memory: Option<Vec<(String, MemHarvest)>>,
    pub temperature: Option<Vec<TempHarvest>>,
    pub procs: Option<(u64, Vec<HashMap<u32, (u64, u32)>>)>,
}

/// Returns the GPU data from NVIDIA cards.
#[inline]
pub fn get_nvidia_vecs(
    temp_type: &TemperatureType, filter: &Option<Filter>, widgets_to_harvest: &UsedWidgets,
) -> Option<GpusData> {
    if let Ok(nvml) = &*NVML_DATA {
        if let Ok(num_gpu) = nvml.device_count() {
            let mut temp_vec = Vec::with_capacity(num_gpu as usize);
            let mut mem_vec = Vec::with_capacity(num_gpu as usize);
            let mut proc_vec = Vec::with_capacity(num_gpu as usize);
            let mut total_mem = 0;
            for i in 0..num_gpu {
                if let Ok(device) = nvml.device_by_index(i) {
                    if let Ok(name) = device.name() {
                        if widgets_to_harvest.use_mem {
                            if let Ok(mem) = device.memory_info() {
                                mem_vec.push((
                                    name.clone(),
                                    MemHarvest {
                                        total_bytes: mem.total,
                                        used_bytes: mem.used,
                                        use_percent: if mem.total == 0 {
                                            None
                                        } else {
                                            Some(mem.used as f64 / mem.total as f64 * 100.0)
                                        },
                                    },
                                ));
                            }
                        }
                        if widgets_to_harvest.use_temp && is_temp_filtered(filter, &name) {
                            if let Ok(temperature) = device.temperature(TemperatureSensor::Gpu) {
                                let temperature = temp_type.convert_temp_unit(temperature as f32);

                                temp_vec.push(TempHarvest {
                                    name: name.clone(),
                                    temperature,
                                });
                            }
                        }
                    }
                    if widgets_to_harvest.use_proc {
                        let mut procs = HashMap::new();
                        if let Ok(gpu_procs) = device.process_utilization_stats(None) {
                            for proc in gpu_procs {
                                let pid = proc.pid;
                                let gpu_util = proc.sm_util + proc.enc_util + proc.dec_util;
                                procs.insert(pid, (0, gpu_util));
                            }
                        }
                        if let Ok(compute_procs) = device.running_compute_processes() {
                            for proc in compute_procs {
                                let pid = proc.pid;
                                let gpu_mem = match proc.used_gpu_memory {
                                    UsedGpuMemory::Used(val) => val,
                                    UsedGpuMemory::Unavailable => 0,
                                };
                                if let Some(prev) = procs.get(&pid) {
                                    procs.insert(pid, (gpu_mem, prev.1));
                                } else {
                                    procs.insert(pid, (gpu_mem, 0));
                                }
                            }
                        }
                        // Use the legacy API too but prefer newer API results
                        if let Ok(graphics_procs) = device.running_graphics_processes_v2() {
                            for proc in graphics_procs {
                                let pid = proc.pid;
                                let gpu_mem = match proc.used_gpu_memory {
                                    UsedGpuMemory::Used(val) => val,
                                    UsedGpuMemory::Unavailable => 0,
                                };
                                if let Some(prev) = procs.get(&pid) {
                                    procs.insert(pid, (gpu_mem, prev.1));
                                } else {
                                    procs.insert(pid, (gpu_mem, 0));
                                }
                            }
                        }
                        if let Ok(graphics_procs) = device.running_graphics_processes() {
                            for proc in graphics_procs {
                                let pid = proc.pid;
                                let gpu_mem = match proc.used_gpu_memory {
                                    UsedGpuMemory::Used(val) => val,
                                    UsedGpuMemory::Unavailable => 0,
                                };
                                if let Some(prev) = procs.get(&pid) {
                                    procs.insert(pid, (gpu_mem, prev.1));
                                } else {
                                    procs.insert(pid, (gpu_mem, 0));
                                }
                            }
                        }
                        if !procs.is_empty() {
                            proc_vec.push(procs);
                        }
                        // running total for proc %
                        if let Ok(mem) = device.memory_info() {
                            total_mem += mem.total;
                        }
                    }
                }
            }
            Some(GpusData {
                memory: if !mem_vec.is_empty() {
                    Some(mem_vec)
                } else {
                    None
                },
                temperature: if !temp_vec.is_empty() {
                    Some(temp_vec)
                } else {
                    None
                },
                procs: if !proc_vec.is_empty() {
                    Some((total_mem, proc_vec))
                } else {
                    None
                },
            })
        } else {
            None
        }
    } else {
        None
    }
}
