use std::{num::NonZeroU64, sync::OnceLock};

use nvml_wrapper::{
    Nvml, enum_wrappers::device::TemperatureSensor, enums::device::UsedGpuMemory, error::NvmlError,
};

use crate::{
    app::filter::Filter,
    collection::{DataCollector, memory::MemData, processes::Pid, temperature::TempSensorData},
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

/// Returns whether the vendor ID passed in is NVIDIA's vendor ID.
///
/// See <https://raw.githubusercontent.com/torvalds/linux/master/include/linux/pci_ids.h> for details
/// (search for `PCI_VENDOR_ID_NVIDIA`).
#[cfg(target_os = "linux")]
#[inline]
fn is_nvidia_vendor(vendor_id: &str) -> bool {
    const NVIDIA_VENDOR: &str = "0x10de";
    vendor_id == NVIDIA_VENDOR
}

/// Returns whether the PCI code is a GPU.
///
/// See <https://raw.githubusercontent.com/torvalds/linux/master/include/linux/pci_ids.h> for details
/// (search for `PCI_BASE_CLASS_DISPLAY`).
#[cfg(target_os = "linux")]
#[inline]
fn is_gpu_class(class_code: &str) -> bool {
    const PCI_BASE_CLASS_DISPLAY: &str = "0x03";
    class_code.starts_with(PCI_BASE_CLASS_DISPLAY)
}

/// Get a list of PCI bus IDs for Linux. This will handle whether the device is awake or not.
/// We do this separately to avoid the possibility of NVML waking up the device at all;
/// this is particularly useful for things like laptops with hybrid graphics (e.g. NVIDIA Optimus).
///
/// Note this is somewhat expensive, so it may be worth caching this result.
///
/// ---
///
/// For more information, see:
/// - <https://us.download.nvidia.com/XFree86/Linux-x86_64/525.89.02/README/dynamicpowermanagement.html>
/// - <https://www.kernel.org/doc/Documentation/ABI/testing/sysfs-devices-power_state>
#[cfg(target_os = "linux")]
fn get_active_pci_bus_ids() -> Vec<String> {
    use crate::collection::linux::utils::is_device_awake;
    use std::fs;

    let Ok(entries) = fs::read_dir("/sys/bus/pci/devices") else {
        return Vec::new();
    };

    let mut result: Vec<String> = entries
        .flatten()
        .filter_map(|entry| {
            let path = entry.path();

            let is_nvidia = fs::read_to_string(path.join("vendor"))
                .is_ok_and(|vendor| is_nvidia_vendor(vendor.trim()));
            if !is_nvidia {
                return None;
            }

            let is_gpu = fs::read_to_string(path.join("class"))
                .is_ok_and(|class| is_gpu_class(class.trim()));
            if !is_gpu {
                return None;
            }

            let is_awake = is_device_awake(&path);

            // This returns values in the "shape" of "0000:01:00.0" (domain:bus:device.function).
            //
            // Just as an FYI:
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
            if is_awake {
                // Note that NVML expects an eight-digit bus ID at the front, so we prepend the current device name
                // with `0000`.
                entry
                    .file_name()
                    .into_string()
                    .ok()
                    .map(|name| concat_string::concat_string!("0000", name))
            } else {
                None
            }
        })
        .collect();

    result.sort_unstable();
    result
}

/// Returns the GPU data from NVIDIA cards.
#[inline]
pub fn get_nvidia_gpu_data(collector: &DataCollector) -> Option<GpusData> {
    let filter = &collector.filters.temp_filter;
    let graph_filter = &collector.filters.temp_graph_filter;
    let widgets_to_harvest = &collector.widgets_to_harvest;

    let Ok(nvml) = NVML_DATA.get_or_init(init_nvml) else {
        return None;
    };

    let (gpu_iter, max_num_gpus): (_, usize) = {
        cfg_select! {
            target_os = "linux" => {
                // TODO: Cache this, maybe every 10s? Want to avoid calling this too often as it may be expensive. Could also cache w/ number of entries to auto-bust?
                let pci_bus_ids = get_active_pci_bus_ids();
                let num_gpus = pci_bus_ids.len();

                (pci_bus_ids.into_iter().filter_map(|id| nvml.device_by_pci_bus_id(id).ok()), num_gpus)
            },
            _ => {
                // The fallback behaviour (the old one) is to just list all nvml devices blindly.
                // Note this has the risk of waking up sleeping devices.
                let num_gpus = nvml.device_count().ok()?;
                ((0..num_gpus).map(|i| nvml.device_by_index(i)).flatten(), num_gpus as usize)
            }
        }
    };

    let mut temp_vec = Vec::with_capacity(max_num_gpus);
    let mut mem_vec = Vec::with_capacity(max_num_gpus);
    let mut proc_vec = Vec::with_capacity(max_num_gpus);
    let mut total_mem = 0;

    for device in gpu_iter {
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
}
