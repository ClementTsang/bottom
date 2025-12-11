use std::{num::NonZeroU64, sync::OnceLock};

use hashbrown::HashMap;
use nvml_wrapper::{
    Nvml, enum_wrappers::device::TemperatureSensor, enums::device::UsedGpuMemory, error::NvmlError,
};

use crate::{
    app::{filter::Filter, layout_manager::UsedWidgets},
    collection::{memory::MemData, temperature::TempSensorData},
};

pub static NVML_DATA: OnceLock<Result<Nvml, NvmlError>> = OnceLock::new();

pub struct GpusData {
    pub memory: Option<Vec<(String, MemData)>>,
    pub temperature: Option<Vec<TempSensorData>>,
    pub procs: Option<(u64, Vec<HashMap<u32, (u64, u32)>>)>,
}

/// Wrapper around Nvml::init
///
/// On Linux, if `Nvml::init()` fails, this function attempts to explicitly load
/// the library from `libnvidia-ml.so.1`. On other platforms, it simply calls `Nvml::init`.
///
/// This is a workaround until https://github.com/Cldfire/nvml-wrapper/pull/63 is accepted.
/// Then, we can go back to calling `Nvml::init` directly on all platforms.
fn init_nvml() -> Result<Nvml, NvmlError> {
    #[cfg(not(target_os = "linux"))]
    let res = Nvml::init();

    #[cfg(target_os = "linux")]
    let res = match Nvml::init() {
        Ok(nvml) => Ok(nvml),
        Err(_) => Nvml::builder()
            .lib_path(std::ffi::OsStr::new("libnvidia-ml.so.1"))
            .init(),
    };

    if let Err(_e) = &res {
        crate::error!("Failed to initialize NVML: {_e}");
    }

    res
}

/// Returns the GPU data from NVIDIA cards.
#[inline]
pub fn get_nvidia_vecs(
    filter: &Option<Filter>, widgets_to_harvest: &UsedWidgets,
) -> Option<GpusData> {
    if let Ok(nvml) = NVML_DATA.get_or_init(init_nvml) {
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
                                if let Some(total_bytes) = NonZeroU64::new(mem.total) {
                                    mem_vec.push((
                                        name.clone(),
                                        MemData {
                                            total_bytes,
                                            used_bytes: mem.used,
                                        },
                                    ));
                                }
                            }
                        }

                        if widgets_to_harvest.use_temp
                            && Filter::optional_should_keep(filter, &name)
                        {
                            if let Ok(temperature) = device.temperature(TemperatureSensor::Gpu) {
                                temp_vec.push(TempSensorData {
                                    name,
                                    temperature: Some(temperature as f32),
                                });
                            } else {
                                temp_vec.push(TempSensorData {
                                    name,
                                    temperature: None,
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
