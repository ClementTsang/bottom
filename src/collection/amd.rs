mod amd_gpu_marketing;

use std::{
    fs::{self, read_to_string},
    num::NonZeroU64,
    path::{Path, PathBuf},
    sync::{LazyLock, Mutex},
    time::{Duration, Instant},
};

use hashbrown::{HashMap, HashSet};

use super::linux::utils::is_device_awake;
use crate::{app::layout_manager::UsedWidgets, collection::memory::MemData};

// TODO: May be able to clean up some of these, Option<Vec> for example is a bit redundant.
pub struct AmdGpuData {
    pub memory: Option<Vec<(String, MemData)>>,
    pub procs: Option<(u64, Vec<HashMap<u32, (u64, u32)>>)>,
}

pub struct AmdGpuMemory {
    pub total: u64,
    pub used: u64,
}

#[derive(Debug, Clone, Default, Eq, PartialEq)]
pub struct AmdGpuProc {
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
static PROC_DATA: LazyLock<Mutex<HashMap<PathBuf, HashMap<u32, AmdGpuProc>>>> =
    LazyLock::new(|| Mutex::new(HashMap::new()));

fn get_amd_devs() -> Option<Vec<PathBuf>> {
    let mut devices = Vec::new();

    // read all PCI devices controlled by the AMDGPU module
    let Ok(paths) = fs::read_dir("/sys/module/amdgpu/drivers/pci:amdgpu") else {
        return None;
    };

    for path in paths {
        let Ok(path) = path else { continue };

        // test if it has a valid vendor path
        let device_path = path.path();
        if !device_path.is_dir() {
            continue;
        }

        // Skip if asleep to avoid wakeups.
        if !is_device_awake(&device_path) {
            continue;
        }

        // This will exist for GPUs but not others, this is how we find their kernel
        // name.
        let test_path = device_path.join("drm");
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
    amd_gpu_marketing::AMD_GPU_MARKETING_NAME
        .iter()
        .find(|(did, rid, _)| (did, rid) == (&device_id, &revision_id))
        .map(|tuple| tuple.2.to_string())
}

fn get_amd_vram(device_path: &Path) -> Option<AmdGpuMemory> {
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

    Some(AmdGpuMemory {
        total: vram_total,
        used: vram_used,
    })
}

// from amdgpu_top: https://github.com/Umio-Yasuno/amdgpu_top/blob/c961cf6625c4b6d63fda7f03348323048563c584/crates/libamdgpu_top/src/stat/fdinfo/proc_info.rs#L114
fn diff_usage(pre: u64, cur: u64, interval: &Duration) -> u64 {
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
fn get_amdgpu_pid_fds(pid: u32, device_path: Vec<PathBuf>) -> Option<Vec<u32>> {
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

fn get_amdgpu_drm(device_path: &Path) -> Option<Vec<PathBuf>> {
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

fn get_amd_fdinfo(device_path: &Path) -> Option<HashMap<u32, AmdGpuProc>> {
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

        let mut usage: AmdGpuProc = Default::default();

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
                    "drm-memory-vram" => usage.vram_usage += fdinfo_value_num << 10, // KiB -> B
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

pub fn get_amd_vecs(widgets_to_harvest: &UsedWidgets, prev_time: Instant) -> Option<AmdGpuData> {
    let device_path_list = get_amd_devs()?;
    let interval = Instant::now().duration_since(prev_time);
    let num_gpu = device_path_list.len();
    let mut mem_vec = Vec::with_capacity(num_gpu);
    let mut proc_vec = Vec::with_capacity(num_gpu);
    let mut total_mem = 0;

    for device_path in device_path_list {
        let device_name = get_amd_name(&device_path)
            .unwrap_or(amd_gpu_marketing::AMDGPU_DEFAULT_NAME.to_string());

        if let Some(mem) = get_amd_vram(&device_path) {
            if widgets_to_harvest.use_mem {
                if let Some(total_bytes) = NonZeroU64::new(mem.total) {
                    mem_vec.push((
                        device_name.clone(),
                        MemData {
                            total_bytes,
                            used_bytes: mem.used,
                        },
                    ));
                }
            }

            total_mem += mem.total
        }

        if widgets_to_harvest.use_proc {
            if let Some(procs) = get_amd_fdinfo(&device_path) {
                let mut proc_info = PROC_DATA.lock().unwrap();
                let _ = proc_info.try_insert(device_path.clone(), HashMap::new());
                let prev_fdinfo = proc_info.get_mut(&device_path).unwrap();

                let mut procs_map = HashMap::new();
                for (proc_pid, proc_usage) in procs {
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

                        if gpu_util > 0 || proc_usage.vram_usage > 0 {
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

    Some(AmdGpuData {
        memory: (!mem_vec.is_empty()).then_some(mem_vec),
        procs: (!proc_vec.is_empty()).then_some((total_mem, proc_vec)),
    })
}
