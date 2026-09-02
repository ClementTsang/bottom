mod amd_gpu_marketing;

use std::{
    cell::RefCell,
    fs::read_to_string,
    num::NonZeroU64,
    path::{Path, PathBuf},
    time::Instant,
};

use rustc_hash::{FxHashMap as HashMap, FxHashSet as HashSet};

use crate::{
    app::layout_manager::UsedWidgets,
    collection::{
        linux::drm::{collect_drm_fdinfo, diff_usage, enumerate_drm_devices, get_drm_render_nodes},
        memory::MemData,
        processes::Pid,
    },
    utils::int_hash::{IntHashMap, IntHashSet},
};

// TODO: May be able to clean up some of these, Option<Vec> for example is a bit
// redundant.
pub struct AmdGpuData {
    pub memory: Option<Vec<(String, MemData)>>,
    pub procs: Option<(u64, Vec<IntHashMap<Pid, (u64, u32)>>)>,
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

// TODO: This is kind of a hack.
thread_local! {
    static PREV_PROC_DATA: RefCell<HashMap<PathBuf, IntHashMap<Pid, AmdGpuProc>>> = RefCell::new(HashMap::default());
    static LAST_CLEAN_COUNTER: RefCell<u32> = const { RefCell::new(0) };
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

    if let Some(stripped) = rev_data.strip_prefix("0x") {
        rev_data = stripped.to_string();
    }

    if let Some(stripped) = dev_data.strip_prefix("0x") {
        dev_data = stripped.to_string();
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

fn get_amd_fdinfo(device_path: &Path) -> Option<IntHashMap<Pid, AmdGpuProc>> {
    let drm_paths = get_drm_render_nodes(device_path)?;

    collect_drm_fdinfo(
        &drm_paths,
        |usage: &mut AmdGpuProc, (keyword, value)| match keyword {
            "drm-engine-gfx" => usage.gfx_usage += value,
            "drm-engine-dma" => usage.dma_usage += value,
            "drm-engine-dec" => usage.dec_usage += value,
            "drm-engine-enc" => usage.enc_usage += value,
            "drm-engine-enc_1" => usage.uvd_usage += value,
            "drm-engine-jpeg" => usage.vcn_usage += value,
            "drm-engine-vpe" => usage.vpe_usage += value,
            "drm-engine-compute" => usage.compute_usage += value,
            "drm-memory-vram" => usage.vram_usage += value << 10, // KiB -> B
            _ => {}
        },
    )
}

pub fn get_amd_vecs(widgets_to_harvest: &UsedWidgets, prev_time: Instant) -> Option<AmdGpuData> {
    let device_path_list = enumerate_drm_devices("amdgpu")?;
    let interval = Instant::now().duration_since(prev_time);
    let num_gpu = device_path_list.len();
    let mut mem_vec = Vec::with_capacity(num_gpu);
    let mut proc_vec = Vec::with_capacity(num_gpu);
    let mut total_mem = 0;

    PREV_PROC_DATA.with_borrow_mut(|prev_proc_data| {
        let device_path_set = device_path_list.iter().cloned().collect::<HashSet<_>>();
        prev_proc_data.retain(|k, _| device_path_set.contains(k));
    });

    for device_path in device_path_list {
        let device_name = get_amd_name(&device_path)
            .unwrap_or(amd_gpu_marketing::AMDGPU_DEFAULT_NAME.to_string());

        if let Some(mem) = get_amd_vram(&device_path) {
            if widgets_to_harvest.use_mem
                && let Some(total_bytes) = NonZeroU64::new(mem.total)
            {
                mem_vec.push((
                    device_name.clone(),
                    MemData {
                        total_bytes,
                        used_bytes: mem.used,
                    },
                ));
            }

            total_mem += mem.total
        }

        if widgets_to_harvest.use_proc
            && let Some(procs) = get_amd_fdinfo(&device_path)
        {
            PREV_PROC_DATA.with_borrow_mut(|prev_proc_data| {
                let prev_fdinfo = prev_proc_data.entry(device_path).or_default();
                let mut seen_pids = IntHashSet::default();

                let mut procs_map = IntHashMap::default();
                for (proc_pid, proc_usage) in procs {
                    seen_pids.insert(proc_pid);
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

                prev_fdinfo.retain(|k, _| seen_pids.contains(k));

                if !procs_map.is_empty() {
                    proc_vec.push(procs_map);
                }
            });
        }
    }

    // Bit of a hacky way to keep this trimmed. Ain't pretty but it should work.
    LAST_CLEAN_COUNTER.with_borrow_mut(|counter| {
        *counter += 1;

        if *counter >= 300 {
            PREV_PROC_DATA.with_borrow_mut(|prev_proc_data| {
                for prev_fdinfo in prev_proc_data.values_mut() {
                    prev_fdinfo.shrink_to_fit();
                }

                prev_proc_data.shrink_to_fit();
            });

            *counter = 0;
        }
    });

    Some(AmdGpuData {
        memory: (!mem_vec.is_empty()).then_some(mem_vec),
        procs: (!proc_vec.is_empty()).then_some((total_mem, proc_vec)),
    })
}
