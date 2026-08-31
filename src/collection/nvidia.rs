use std::{num::NonZeroU64, sync::OnceLock};

use nvml_wrapper::{
    Device, Nvml, enum_wrappers::device::TemperatureSensor, enums::device::UsedGpuMemory,
    error::NvmlError,
};

use crate::{
    app::{filter::Filter, layout_manager::UsedWidgets},
    collection::{memory::MemData, processes::Pid, temperature::TempSensorData},
    utils::int_hash::IntHashMap,
};

pub static NVML_DATA: OnceLock<Result<Nvml, NvmlError>> = OnceLock::new();

pub struct GpusData {
    pub memory: Option<Vec<(String, MemData)>>,
    pub temperature: Option<Vec<TempSensorData>>,
    pub procs: Option<(u64, Vec<IntHashMap<Pid, (u64, u32)>>)>,
}

/// Wrapper around Nvml::init
///
/// On Linux, if `Nvml::init()` fails, this function attempts to explicitly load
/// the library from `libnvidia-ml.so.1`. On other platforms, it simply calls
/// `Nvml::init`.
///
/// This is a workaround until https://github.com/Cldfire/nvml-wrapper/pull/63 is accepted.
/// Then, we can go back to calling `Nvml::init` directly on all platforms.
fn init_nvml() -> Result<Nvml, NvmlError> {
    #[cfg(not(target_os = "linux"))]
    {
        Nvml::init()
    }
    #[cfg(target_os = "linux")]
    {
        match Nvml::init() {
            Ok(nvml) => Ok(nvml),
            Err(_) => Nvml::builder()
                .lib_path(std::ffi::OsStr::new("libnvidia-ml.so.1"))
                .init(),
        }
    }
}

/// Return whether the device (typically a GPU) is awake or not. This is useful for things like laptops that may
/// have hybrid graphics (e.g. NVIDIA Optimus).
///
/// Note that it is possible this check fails; in this case it will return an [`NvmlError`].
///
/// ------
///
/// For Linux, we check things similarly to how it's done with other non-Nvidia devices already, by checking the
/// `power_state` file in sysfs. Note that if the associated device path somehow does not exist, we just assume it
/// is awake for simplicity.
///
/// For more information, see:
/// - <https://us.download.nvidia.com/XFree86/Linux-x86_64/525.89.02/README/dynamicpowermanagement.html>
/// - <https://www.kernel.org/doc/Documentation/ABI/testing/sysfs-devices-power_state>
#[cfg(target_os = "linux")]
#[inline]
fn is_nv_device_awake(device: &Device<'_>) -> Result<bool, NvmlError> {
    use crate::collection::linux::utils::is_device_awake;
    use std::path::PathBuf;

    let pci_info = device.pci_info()?; // TODO: Does this wake up the GPU...?

    // The "0th" function is the GPU itself - from the NVIDIA power management docs
    // (https://us.download.nvidia.com/XFree86/Linux-x86_64/525.89.02/README/dynamicpowermanagement.html):
    // > The NVIDIA GPU may have one, two or four PCI functions:
    // > - Function 0: VGA controller / 3D controller
    // > - Function 1: Audio device
    // > - Function 2: USB xHCI Host controller
    // > - Function 3: USB Type-C UCSI controller
    //
    // We also know the "shape" of the path from aforementioned docs (ignore what it's trying to do):
    // > For pre-Ampere notebooks, runtime D3 power management can be enabled for each PCI function using the following command.
    // > echo auto > /sys/bus/pci/devices/<Domain>:<Bus>:<Device>.<Function>/power/control
    // > For example:
    // > echo auto > /sys/bus/pci/devices/0000:01:00.0/power/control
    let device_path = PathBuf::from(format!(
        "/sys/bus/pci/devices/{:04x}:{:02x}:{:02x}.0",
        pci_info.domain, pci_info.bus, pci_info.device
    ));

    // Not going to even bother checking if the device path exists here, as
    // `is_device_awake` kinda already does that for us.
    Ok(is_device_awake(&device_path))
}

/// Return whether the device (typically a GPU) is awake or not. This is useful for things like laptops that may
/// have hybrid graphics (e.g. NVIDIA Optimus).
///
/// While some variants of this function can fail, for this catch-all implementation, it will always succeed and
/// return `Ok(true)`.
#[cfg(not(target_os = "linux"))]
#[inline]
fn is_nv_device_awake(_device: &Device<'_>) -> Result<bool, NvmlError> {
    Ok(true)
}

/// Returns the GPU data from NVIDIA cards.
#[inline]
pub fn get_nvidia_vecs(
    filter: &Option<Filter>, graph_filter: &Option<Filter>, widgets_to_harvest: &UsedWidgets,
) -> Option<GpusData> {
    if let Ok(nvml) = NVML_DATA.get_or_init(init_nvml) {
        if let Ok(num_gpu) = nvml.device_count() {
            let mut temp_vec = Vec::with_capacity(num_gpu as usize);
            let mut mem_vec = Vec::with_capacity(num_gpu as usize);
            let mut proc_vec = Vec::with_capacity(num_gpu as usize);
            let mut total_mem = 0;

            for i in 0..num_gpu {
                if let Ok(device) = nvml.device_by_index(i) {
                    // Skip to avoid waking up the GPU. If we can't determine it, we just default to being awake.
                    if !is_nv_device_awake(&device).unwrap_or(true) {
                        continue;
                    }

                    if let Ok(name) = device.name() {
                        if widgets_to_harvest.use_mem
                            && let Ok(mem) = device.memory_info()
                            && let Some(total_bytes) = NonZeroU64::new(mem.total)
                        {
                            mem_vec.push((
                                name.clone(),
                                MemData {
                                    total_bytes,
                                    used_bytes: mem.used,
                                },
                            ));
                        }

                        if (widgets_to_harvest.use_temp || widgets_to_harvest.use_temp_graph)
                            && (Filter::optional_should_keep(filter, &name)
                                || Filter::optional_should_keep(graph_filter, &name))
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
                        let mut procs = IntHashMap::default();

                        if let Ok(gpu_procs) = device.process_utilization_stats(None) {
                            for proc in gpu_procs {
                                let pid = proc.pid as Pid;
                                let gpu_util = proc.sm_util + proc.enc_util + proc.dec_util;
                                procs.insert(pid, (0, gpu_util));
                            }
                        }

                        if let Ok(compute_procs) = device.running_compute_processes() {
                            for proc in compute_procs {
                                let pid = proc.pid as Pid;
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
                                let pid = proc.pid as Pid;
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
                                let pid = proc.pid as Pid;
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
