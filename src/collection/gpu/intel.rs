//! A collection of functions/utilities to get Intel GPU data.
//!
//! Note this currently only works on Linux. For Linux, we support both the `i915` and `xe` DRM drivers.
//! For more info, see <https://docs.kernel.org/gpu/drm-usage-stats.html> for info on the fdinfo format.

use std::{
    cell::RefCell,
    num::NonZeroU64,
    path::{Path, PathBuf},
    time::{Duration, Instant},
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

/// Which DRM driver backs a given Intel device.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum IntelDriver {
    I915,
    Xe,
}

impl IntelDriver {
    /// The kernel module name, i.e. the `/sys/module/<name>` directory.
    const fn module(self) -> &'static str {
        match self {
            IntelDriver::I915 => "i915",
            IntelDriver::Xe => "xe",
        }
    }
}

/// Intel GPU data.
///
/// TODO: May be able to clean up some of these, Option<Vec> for example is a bit
/// redundant.
pub struct IntelGpuData {
    pub memory: Option<Vec<(String, MemData)>>,
    pub process_data: Option<(u64, Vec<IntHashMap<Pid, (u64, u32)>>)>,
}

/// Intel GPU VRAM usage.
pub struct IntelGpuMemory {
    pub total: u64,
    pub used: u64,
}

/// Per-process accumulator. It holds fields for *both* drivers; only the subset relevant to the
/// device's driver is ever populated (see [`get_intel_fdinfo`]).
#[derive(Debug, Clone, Default, Eq, PartialEq)]
struct IntelGpuProc {
    // i915: per-engine busy time, in nanoseconds (accumulating counters).
    render_usage: u64,
    copy_usage: u64,
    video_usage: u64,
    video_enhance_usage: u64,
    compute_usage: u64,
    // xe: busy + elapsed cycle counters, summed across all engines.
    cycles: u64,
    total_cycles: u64,
    // Per-process GPU memory, in bytes.
    mem_used: u64,
}

// TODO: This is kind of a hack.
thread_local! {
    static PREV_PROC_DATA: RefCell<HashMap<PathBuf, IntHashMap<Pid, IntelGpuProc>>> = RefCell::new(HashMap::default());
    static LAST_CLEAN_COUNTER: RefCell<u32> = const { RefCell::new(0) };
}

/// Apparently there's no marketing name to grab...?
///
/// TODO: Investigate this properly.
fn get_intel_name(num_gpu: usize, idx: usize) -> String {
    const INTEL_DEFAULT_NAME: &str = "Intel GPU";

    if num_gpu > 1 {
        format!("{INTEL_DEFAULT_NAME} {idx}")
    } else {
        INTEL_DEFAULT_NAME.to_string()
    }
}

/// VRAM used for a device.
fn get_intel_vram(_device_path: &Path, _driver: IntelDriver) -> Option<IntelGpuMemory> {
    // TODO: Not yet implemented.
    None
}

fn get_intel_fdinfo(
    device_path: &Path, driver: IntelDriver,
) -> Option<IntHashMap<Pid, IntelGpuProc>> {
    let drm_paths = get_drm_render_nodes(device_path)?;

    match driver {
        IntelDriver::I915 => {
            collect_drm_fdinfo(&drm_paths, |usage: &mut IntelGpuProc, (keyword, value)| {
                match keyword {
                    "drm-engine-render" => usage.render_usage += value,
                    "drm-engine-copy" => usage.copy_usage += value,
                    "drm-engine-video" => usage.video_usage += value,
                    "drm-engine-video-enhance" => usage.video_enhance_usage += value,
                    "drm-engine-compute" => usage.compute_usage += value,
                    // Memory keys are in KiB: local0 = discrete VRAM, system0 = shared system RAM.
                    "drm-total-local0" | "drm-total-system0" => usage.mem_used += value << 10,
                    _ => {}
                }
            })
        }
        IntelDriver::Xe => {
            collect_drm_fdinfo(&drm_paths, |usage: &mut IntelGpuProc, (keyword, value)| {
                match keyword {
                    // rcs = render, ccs = compute, vcs = video decode, vecs = video enhance, bcs = copy.
                    "drm-cycles-rcs" | "drm-cycles-ccs" | "drm-cycles-vcs" | "drm-cycles-vecs"
                    | "drm-cycles-bcs" => usage.cycles += value,
                    "drm-total-cycles-rcs"
                    | "drm-total-cycles-ccs"
                    | "drm-total-cycles-vcs"
                    | "drm-total-cycles-vecs"
                    | "drm-total-cycles-bcs" => usage.total_cycles += value,
                    // Memory key is in KiB.
                    "drm-total-vram0" => usage.mem_used += value << 10,
                    _ => {}
                }
            })
        }
    }
}

fn compute_util(
    driver: IntelDriver, prev: &IntelGpuProc, cur: &IntelGpuProc, interval: &Duration,
) -> u32 {
    match driver {
        // Busy-time deltas over the interval, summed across engines (as AMD does).
        IntelDriver::I915 => {
            let util = diff_usage(prev.render_usage, cur.render_usage, interval)
                + diff_usage(prev.copy_usage, cur.copy_usage, interval)
                + diff_usage(prev.video_usage, cur.video_usage, interval)
                + diff_usage(prev.video_enhance_usage, cur.video_enhance_usage, interval)
                + diff_usage(prev.compute_usage, cur.compute_usage, interval);

            util.try_into().unwrap_or(0)
        }
        // Ratio of busy cycles to elapsed cycles over the interval.
        IntelDriver::Xe => {
            let cycles_delta = cur.cycles.saturating_sub(prev.cycles);
            let total_delta = cur.total_cycles.saturating_sub(prev.total_cycles);

            if prev.total_cycles == 0 || total_delta == 0 {
                0
            } else {
                cycles_delta
                    .saturating_mul(100)
                    .checked_div(total_delta)
                    .unwrap_or(0)
                    .try_into()
                    .unwrap_or(0)
            }
        }
    }
}

pub fn get_intel_gpu_data(
    widgets_to_harvest: &UsedWidgets, prev_time: Instant,
) -> Option<IntelGpuData> {
    // TODO: Add caching for this.
    let mut i915_devices: Vec<PathBuf> = vec![];
    let mut xe_devices: Vec<PathBuf> = vec![];

    if let Some(paths) = enumerate_drm_devices(IntelDriver::I915.module()) {
        i915_devices.extend(paths);
    }

    if let Some(paths) = enumerate_drm_devices(IntelDriver::Xe.module()) {
        xe_devices.extend(paths);
    }

    if i915_devices.is_empty() && xe_devices.is_empty() {
        return None;
    }

    let interval = Instant::now().duration_since(prev_time);
    let num_gpus = i915_devices.len() + xe_devices.len();
    let mut mem_vec = Vec::with_capacity(num_gpus);
    let mut proc_vec = Vec::with_capacity(num_gpus);
    let mut total_mem = 0;

    PREV_PROC_DATA.with_borrow_mut(|prev_proc_data| {
        let device_path_set = i915_devices
            .iter()
            .chain(xe_devices.iter())
            .collect::<HashSet<_>>();
        prev_proc_data.retain(|k, _| device_path_set.contains(k));
    });

    let devices = [
        (IntelDriver::I915, i915_devices),
        (IntelDriver::Xe, xe_devices),
    ];

    for (idx, (device_path, driver)) in devices
        .into_iter()
        .flat_map(|(driver, devices)| {
            devices
                .into_iter()
                .map(move |device_path| (device_path, driver))
        })
        .enumerate()
    {
        let device_name = get_intel_name(num_gpus, idx);

        if let Some(mem) = get_intel_vram(&device_path, driver) {
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

            total_mem += mem.total;
        }

        if widgets_to_harvest.use_proc
            && let Some(procs) = get_intel_fdinfo(&device_path, driver)
        {
            PREV_PROC_DATA.with_borrow_mut(|prev_proc_data| {
                let prev_fdinfo = prev_proc_data.entry(device_path).or_default();
                let mut seen_pids = IntHashSet::default();

                let mut procs_map = IntHashMap::default();
                for (proc_pid, proc_usage) in procs {
                    seen_pids.insert(proc_pid);
                    if let Some(prev_usage) = prev_fdinfo.get_mut(&proc_pid) {
                        let gpu_util = compute_util(driver, prev_usage, &proc_usage, &interval);

                        if gpu_util > 0 || proc_usage.mem_used > 0 {
                            procs_map.insert(proc_pid, (proc_usage.mem_used, gpu_util));
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

    Some(IntelGpuData {
        memory: (!mem_vec.is_empty()).then_some(mem_vec),
        process_data: (!proc_vec.is_empty()).then_some((total_mem, proc_vec)),
    })
}
