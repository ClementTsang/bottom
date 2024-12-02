mod amdgpu_marketing;

use crate::{
    app::{filter::Filter, layout_manager::UsedWidgets},
    data_collection::{
        memory::MemHarvest,
        temperature::{TempHarvest, TemperatureType},
    },
};
use hashbrown::{HashMap, HashSet};
use std::{
    fs,
    fs::read_to_string,
    path::{Path, PathBuf},
    sync::{LazyLock, Mutex},
    time::{Duration, Instant},
};

pub struct AMDGPUData {
    pub memory: Option<Vec<(String, MemHarvest)>>,
    pub temperature: Option<Vec<TempHarvest>>,
    pub procs: Option<(u64, Vec<HashMap<u32, (u64, u32)>>)>,
}

pub struct AMDGPUMemory {
    pub total: u64,
    pub used: u64,
}

pub struct AMDGPUTemperature {
    pub name: String,
    pub temperature: f32,
}

#[derive(Debug, Clone, Default, Eq, PartialEq)]
pub struct AMDGPUProc {
    pub vram_usage: u64,
    pub gfx_usage: u64,
    pub dma_usage: u64,
    pub enc_usage: u64,
    pub dec_usage: u64,
    pub uvd_usage: u64,
    pub vcn_usage: u64,
    pub vpe_usage: u64,
    pub compute_usage: u64,
}

// needs previous state for usage calculation
static PROC_DATA: LazyLock<Mutex<HashMap<PathBuf, HashMap<u32, AMDGPUProc>>>> =
    LazyLock::new(|| Mutex::new(HashMap::new()));

pub fn get_amd_devs() -> Option<Vec<PathBuf>> {
    let mut devices = Vec::new();

    // read all PCI devices controlled b y the AMDGPU module
    let Ok(paths) = fs::read_dir("/sys/module/amdgpu/drivers/pci:amdgpu") else {
        return None;
    };

    for path in paths {
        let Ok(path) = path else { continue };

        // test if it has a valid vendor path
        let device_path = path.path();
        let test_path = device_path.join("vendor");
        if test_path.as_path().exists() {
            devices.push(device_path);
        }
    }

    if devices.is_empty() {
        None
    } else {
        Some(devices)
    }
}

pub fn get_amd_name(device_path: &Path) -> Option<String> {
    // get revision and device ids from sysfs
    let rev_path = device_path.join("revision");
    let dev_path = device_path.join("device");

    if !rev_path.exists() || !dev_path.exists() {
        return None;
    }

    // read and remove newlines, 0x0 suffix.
    let mut rev_data = read_to_string(rev_path).unwrap_or("0x00".to_string());
    let mut dev_data = read_to_string(dev_path).unwrap_or("0x0000".to_string());

    rev_data = rev_data.trim_end().to_string();
    dev_data = dev_data.trim_end().to_string();

    if rev_data.starts_with("0x") {
        rev_data = rev_data.strip_prefix("0x").unwrap().to_string();
    }

    if dev_data.starts_with("0x") {
        dev_data = dev_data.strip_prefix("0x").unwrap().to_string();
    }

    let revision_id = u32::from_str_radix(&rev_data, 16).unwrap_or(0);
    let device_id = u32::from_str_radix(&dev_data, 16).unwrap_or(0);

    if device_id == 0 {
        return None;
    }

    // if it exists in our local database, use that name
    amdgpu_marketing::AMDGPU_MARKETING_NAME
        .iter()
        .find(|(did, rid, _)| (did, rid) == (&device_id, &revision_id))
        .map(|tuple| tuple.2.to_string())
}

pub fn get_amd_vram(device_path: &Path) -> Option<AMDGPUMemory> {
    // get vram memory info from sysfs
    let vram_total_path = device_path.join("mem_info_vram_total");
    let vram_used_path = device_path.join("mem_info_vram_used");

    let Ok(mut vram_total_data) = read_to_string(vram_total_path) else {
        return None;
    };
    let Ok(mut vram_used_data) = read_to_string(vram_used_path) else {
        return None;
    };

    // read and remove newlines
    vram_total_data = vram_total_data.trim_end().to_string();
    vram_used_data = vram_used_data.trim_end().to_string();

    let Ok(vram_total) = vram_total_data.parse::<u64>() else {
        return None;
    };
    let Ok(vram_used) = vram_used_data.parse::<u64>() else {
        return None;
    };

    Some(AMDGPUMemory {
        total: vram_total,
        used: vram_used,
    })
}

pub fn get_amd_temp(device_path: &Path) -> Option<Vec<AMDGPUTemperature>> {
    let mut temperatures = Vec::new();

    // get hardware monitoring sensor info from sysfs
    let hwmon_root = device_path.join("hwmon");

    let Ok(hwmon_paths) = fs::read_dir(hwmon_root) else {
        return None;
    };

    for hwmon_dir in hwmon_paths {
        let Ok(hwmon_dir) = hwmon_dir else {
            continue;
        };

        let hwmon_binding = hwmon_dir.path();
        let hwmon_path = hwmon_binding.as_path();
        let Ok(hwmon_sensors) = fs::read_dir(hwmon_path) else {
            continue;
        };

        for hwmon_sensor_ent in hwmon_sensors {
            let Ok(hwmon_sensor_ent) = hwmon_sensor_ent else {
                continue;
            };

            let hwmon_sensor_path = hwmon_sensor_ent.path();
            let hwmon_sensor_binding = hwmon_sensor_ent.file_name();
            let Some(hwmon_sensor_name) = hwmon_sensor_binding.to_str() else {
                continue;
            };

            // temperature sensors are temp{number}_{input,label}
            if !hwmon_sensor_name.starts_with("temp") || !hwmon_sensor_name.ends_with("_input") {
                continue; // filename does not start with temp or ends with input
            }

            // construct label path
            let hwmon_sensor_label_name = hwmon_sensor_name.replace("_input", "_label");
            let hwmon_sensor_label_path = hwmon_path.join(hwmon_sensor_label_name);

            // read and remove newlines
            let Ok(mut hwmon_sensor_data) = read_to_string(hwmon_sensor_path) else {
                continue;
            };

            let Ok(mut hwmon_sensor_label) = read_to_string(hwmon_sensor_label_path) else {
                continue;
            };

            hwmon_sensor_data = hwmon_sensor_data.trim_end().to_string();
            hwmon_sensor_label = hwmon_sensor_label.trim_end().to_string();

            let Ok(hwmon_sensor) = hwmon_sensor_data.parse::<u64>() else {
                continue;
            };

            // uppercase first character
            if hwmon_sensor_label.is_ascii() {
                let (hwmon_sensor_label_head, hwmon_sensor_label_tail) =
                    hwmon_sensor_label.split_at(1);

                hwmon_sensor_label =
                    hwmon_sensor_label_head.to_uppercase() + hwmon_sensor_label_tail;
            }

            // 1 C is reported as 1000
            temperatures.push(AMDGPUTemperature {
                name: hwmon_sensor_label,
                temperature: (hwmon_sensor as f32) / 1000.0f32,
            });
        }
    }

    if temperatures.is_empty() {
        None
    } else {
        Some(temperatures)
    }
}

// from amdgpu_top: https://github.com/Umio-Yasuno/amdgpu_top/blob/c961cf6625c4b6d63fda7f03348323048563c584/crates/libamdgpu_top/src/stat/fdinfo/proc_info.rs#L114
pub fn diff_usage(pre: u64, cur: u64, interval: &Duration) -> u64 {
    use std::ops::Mul;

    let diff_ns = if pre == 0 || cur < pre {
        return 0;
    } else {
        cur.saturating_sub(pre) as u128
    };

    diff_ns
        .mul(100)
        .checked_div(interval.as_nanos())
        .unwrap_or(0) as u64
}

// from amdgpu_top: https://github.com/Umio-Yasuno/amdgpu_top/blob/c961cf6625c4b6d63fda7f03348323048563c584/crates/libamdgpu_top/src/stat/fdinfo/proc_info.rs#L13-L27
pub fn get_amdgpu_pid_fds(pid: u32, device_path: Vec<PathBuf>) -> Option<Vec<u32>> {
    let Ok(fd_list) = fs::read_dir(format!("/proc/{pid}/fd/")) else {
        return None;
    };

    let valid_fds: Vec<u32> = fd_list
        .filter_map(|fd_link| {
            let dir_entry = fd_link.map(|fd_link| fd_link.path()).ok()?;
            let link = fs::read_link(&dir_entry).ok()?;

            // e.g. "/dev/dri/renderD128" or "/dev/dri/card0"
            if device_path.iter().any(|path| link.starts_with(path)) {
                dir_entry.file_name()?.to_str()?.parse::<u32>().ok()
            } else {
                None
            }
        })
        .collect();

    if valid_fds.is_empty() {
        None
    } else {
        Some(valid_fds)
    }
}

pub fn get_amdgpu_drm(device_path: &Path) -> Option<Vec<PathBuf>> {
    let mut drm_devices = Vec::new();
    let drm_root = device_path.join("drm");

    let Ok(drm_paths) = fs::read_dir(drm_root) else {
        return None;
    };

    for drm_dir in drm_paths {
        let Ok(drm_dir) = drm_dir else {
            continue;
        };

        // attempt to get the device renderer name
        let drm_name = drm_dir.file_name();
        let Some(drm_name) = drm_name.to_str() else {
            continue;
        };

        // construct driver device path if valid
        if !drm_name.starts_with("card") && !drm_name.starts_with("render") {
            continue;
        }

        drm_devices.push(PathBuf::from(format!("/dev/dri/{drm_name}")));
    }

    if drm_devices.is_empty() {
        None
    } else {
        Some(drm_devices)
    }
}

pub fn get_amd_fdinfo(device_path: &Path) -> Option<HashMap<u32, AMDGPUProc>> {
    let mut fdinfo = HashMap::new();

    let drm_paths = get_amdgpu_drm(device_path)?;

    let Ok(proc_dir) = fs::read_dir("/proc") else {
        return None;
    };

    let pids: Vec<u32> = proc_dir
        .filter_map(|dir_entry| {
            // check if pid is valid
            let dir_entry = dir_entry.ok()?;
            let metadata = dir_entry.metadata().ok()?;

            if !metadata.is_dir() {
                return None;
            }

            let pid = dir_entry.file_name().to_str()?.parse::<u32>().ok()?;

            // skip init process
            if pid == 1 {
                return None;
            }

            Some(pid)
        })
        .collect();

    for pid in pids {
        // collect file descriptors that point to our device renderers
        let Some(fds) = get_amdgpu_pid_fds(pid, drm_paths.clone()) else {
            continue;
        };

        let mut usage: AMDGPUProc = Default::default();

        let mut observed_ids: HashSet<usize> = HashSet::new();

        for fd in fds {
            let fdinfo_path = format!("/proc/{pid}/fdinfo/{fd}");
            let Ok(fdinfo_data) = read_to_string(fdinfo_path) else {
                continue;
            };

            let mut fdinfo_lines = fdinfo_data
                .lines()
                .skip_while(|l| !l.starts_with("drm-client-id"));
            if let Some(id) = fdinfo_lines.next().and_then(|fdinfo_line| {
                const LEN: usize = "drm-client-id:\t".len();
                fdinfo_line.get(LEN..)?.parse().ok()
            }) {
                if !observed_ids.insert(id) {
                    continue;
                }
            } else {
                continue;
            }

            for fdinfo_line in fdinfo_lines {
                let Some(fdinfo_separator_index) = fdinfo_line.find(':') else {
                    continue;
                };

                let (fdinfo_keyword, mut fdinfo_value) =
                    fdinfo_line.split_at(fdinfo_separator_index);
                fdinfo_value = &fdinfo_value[1..];

                fdinfo_value = fdinfo_value.trim();
                if let Some(fdinfo_value_space_index) = fdinfo_value.find(' ') {
                    fdinfo_value = &fdinfo_value[..fdinfo_value_space_index];
                };

                let Ok(fdinfo_value_num) = fdinfo_value.parse::<u64>() else {
                    continue;
                };

                match fdinfo_keyword {
                    "drm-engine-gfx" => usage.gfx_usage += fdinfo_value_num,
                    "drm-engine-dma" => usage.dma_usage += fdinfo_value_num,
                    "drm-engine-dec" => usage.dec_usage += fdinfo_value_num,
                    "drm-engine-enc" => usage.enc_usage += fdinfo_value_num,
                    "drm-engine-enc_1" => usage.uvd_usage += fdinfo_value_num,
                    "drm-engine-jpeg" => usage.vcn_usage += fdinfo_value_num,
                    "drm-engine-vpe" => usage.vpe_usage += fdinfo_value_num,
                    "drm-engine-compute" => usage.compute_usage += fdinfo_value_num,
                    "drm-memory-vram" => usage.vram_usage += fdinfo_value_num << 10,
                    _ => {}
                };
            }
        }

        if usage != Default::default() {
            fdinfo.insert(pid, usage);
        }
    }

    Some(fdinfo)
}

#[inline]
pub fn get_amd_vecs(
    temp_type: &TemperatureType, filter: &Option<Filter>, widgets_to_harvest: &UsedWidgets,
    prev_time: Instant,
) -> Option<AMDGPUData> {
    let device_path_list = get_amd_devs()?;
    let interval = Instant::now().duration_since(prev_time);
    let num_gpu = device_path_list.len();
    let mut temp_vec = Vec::with_capacity(num_gpu);
    let mut mem_vec = Vec::with_capacity(num_gpu);
    let mut proc_vec = Vec::with_capacity(num_gpu);
    let mut total_mem = 0;

    for device_path in device_path_list {
        let device_name =
            get_amd_name(&device_path).unwrap_or(amdgpu_marketing::AMDGPU_DEFAULT_NAME.to_string());

        if widgets_to_harvest.use_mem {
            if let Some(mem) = get_amd_vram(&device_path) {
                mem_vec.push((
                    device_name.clone(),
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

        if widgets_to_harvest.use_temp && Filter::optional_should_keep(filter, &device_name) {
            if let Some(temperatures) = get_amd_temp(&device_path) {
                for info in temperatures {
                    let temperature = temp_type.convert_temp_unit(info.temperature);

                    temp_vec.push(TempHarvest {
                        name: format!("{} {}", device_name, info.name),
                        temperature: Some(temperature),
                    });
                }
            }
        }

        if widgets_to_harvest.use_proc {
            if let Some(procs) = get_amd_fdinfo(&device_path) {
                let mut proc_info = PROC_DATA.lock().unwrap();
                let _ = proc_info.try_insert(device_path.clone(), HashMap::new());
                let prev_fdinfo = proc_info.get_mut(&device_path).unwrap();

                let mut procs_map = HashMap::new();
                for (proc_pid, proc_usage) in procs {
                    total_mem += proc_usage.vram_usage;

                    if let Some(prev_usage) = prev_fdinfo.get_mut(&proc_pid) {
                        // calculate deltas
                        let gfx_usage =
                            diff_usage(prev_usage.gfx_usage, proc_usage.gfx_usage, &interval);
                        let dma_usage =
                            diff_usage(prev_usage.dma_usage, proc_usage.dma_usage, &interval);
                        let enc_usage =
                            diff_usage(prev_usage.enc_usage, proc_usage.enc_usage, &interval);
                        let dec_usage =
                            diff_usage(prev_usage.dec_usage, proc_usage.dec_usage, &interval);
                        let uvd_usage =
                            diff_usage(prev_usage.uvd_usage, proc_usage.uvd_usage, &interval);
                        let vcn_usage =
                            diff_usage(prev_usage.vcn_usage, proc_usage.vcn_usage, &interval);
                        let vpe_usage =
                            diff_usage(prev_usage.vpe_usage, proc_usage.vpe_usage, &interval);

                        // combined usage
                        let gpu_util_wide = gfx_usage
                            + dma_usage
                            + enc_usage
                            + dec_usage
                            + uvd_usage
                            + vcn_usage
                            + vpe_usage;

                        let gpu_util: u32 = gpu_util_wide.try_into().unwrap_or(0);

                        if gpu_util > 0 {
                            procs_map.insert(proc_pid, (proc_usage.vram_usage, gpu_util));
                        }

                        *prev_usage = proc_usage;
                    } else {
                        prev_fdinfo.insert(proc_pid, proc_usage);
                    }
                }

                if !procs_map.is_empty() {
                    proc_vec.push(procs_map);
                }
            }
        }
    }

    Some(AMDGPUData {
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
