//! A collection of functions/utilities to get Intel GPU data.
//!
//! Note this currently only works on Linux. For Linux, we support both the
//! `i915` and `xe` DRM drivers. For more info, see <https://docs.kernel.org/gpu/drm-usage-stats.html> for info on the fdinfo format.

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
/// TODO: May be able to clean up some of these, Option<Vec> for example is a
/// bit redundant.
pub struct IntelGpuData {
    pub memory: Option<Vec<(String, MemData)>>,
    pub process_data: Option<(u64, Vec<IntHashMap<Pid, (u64, u32)>>)>,
    pub overall_utilization: Option<Vec<(String, f32)>>,
}

/// Intel GPU VRAM usage.
pub struct IntelGpuMemory {
    pub total: u64,
    pub used: u64,
}

/// Per-process accumulator. It holds fields for *both* drivers; only the subset
/// relevant to the device's driver is ever populated (see
/// [`get_intel_fdinfo`]).
#[derive(Debug, Clone, Default, Eq, PartialEq)]
struct IntelGpuProc {
    // Per-engine-class busy counters (accumulating). i915 reports nanoseconds,
    // xe reports cycles.
    busy: [u64; NUM_ENGINE_CLASSES],
    // xe: per-engine-class elapsed cycle counters.
    total_cycles: [u64; NUM_ENGINE_CLASSES],
    // Per-engine-class engine count. 0 means unreported, which implies 1.
    capacity: [u64; NUM_ENGINE_CLASSES],
    // Per-process GPU memory, in bytes.
    mem_used: u64,
}

const NUM_ENGINE_CLASSES: usize = 5;

/// i915 fdinfo engine class names, in [`IntelGpuProc`] array order.
const I915_ENGINES: [&str; NUM_ENGINE_CLASSES] =
    ["render", "copy", "video", "video-enhance", "compute"];

/// xe fdinfo engine class names, in [`IntelGpuProc`] array order.
///
/// rcs = render, bcs = copy, vcs = video decode, vecs = video enhance, ccs =
/// compute.
const XE_ENGINES: [&str; NUM_ENGINE_CLASSES] = ["rcs", "bcs", "vcs", "vecs", "ccs"];

impl IntelGpuProc {
    /// Accumulate a per-engine-class fdinfo value, if `keyword` is one.
    fn add_engine_stat(&mut self, engines: &[&str; NUM_ENGINE_CLASSES], keyword: &str, value: u64) {
        let class_of = |prefix: &str| {
            let name = keyword.strip_prefix(prefix)?;
            engines.iter().position(|engine| *engine == name)
        };

        if let Some(class) = class_of("drm-engine-capacity-") {
            self.capacity[class] = self.capacity[class].max(value);
        } else if let Some(class) = class_of("drm-engine-").or_else(|| class_of("drm-cycles-")) {
            self.busy[class] += value;
        } else if let Some(class) = class_of("drm-total-cycles-") {
            // This is a GPU timestamp rather than a per-client counter, so
            // don't sum it across a process's clients.
            self.total_cycles[class] = self.total_cycles[class].max(value);
        }
    }
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

/// The fraction of each engine class's time spent on a client's work between
/// two samples.
fn engine_busy_fractions(
    driver: IntelDriver, prev: &IntelGpuProc, cur: &IntelGpuProc, interval: &Duration,
) -> [f64; NUM_ENGINE_CLASSES] {
    std::array::from_fn(|class| {
        let busy = cur.busy[class].saturating_sub(prev.busy[class]);
        let elapsed = match driver {
            IntelDriver::I915 => interval.as_nanos() as f64,
            IntelDriver::Xe if prev.total_cycles[class] == 0 => 0.0,
            IntelDriver::Xe => {
                cur.total_cycles[class].saturating_sub(prev.total_cycles[class]) as f64
            }
        };

        if elapsed > 0.0 {
            busy as f64 / elapsed
        } else {
            0.0
        }
    })
}

/// Overall GPU utilization as a percentage, taken as the busiest engine class
/// (normalized by the number of engines in that class).
fn busiest_engine_util(
    busy: &[f64; NUM_ENGINE_CLASSES], capacity: &[u64; NUM_ENGINE_CLASSES],
) -> f32 {
    let busiest = busy
        .iter()
        .zip(capacity)
        .map(|(busy, capacity)| busy / *capacity as f64)
        .fold(0.0, f64::max);

    (busiest.clamp(0.0, 1.0) * 100.0) as f32
}

fn get_intel_fdinfo(
    device_path: &Path, driver: IntelDriver,
) -> Option<IntHashMap<Pid, IntelGpuProc>> {
    let drm_paths = get_drm_render_nodes(device_path)?;

    match driver {
        IntelDriver::I915 => {
            collect_drm_fdinfo(&drm_paths, |usage: &mut IntelGpuProc, (keyword, value)| {
                match keyword {
                    // Memory keys are in KiB:
                    // - local0 = discrete VRAM
                    // - system0 = shared system RAM
                    "drm-total-local0" | "drm-total-system0" => usage.mem_used += value << 10,
                    _ => usage.add_engine_stat(&I915_ENGINES, keyword, value),
                }
            })
        }
        IntelDriver::Xe => {
            collect_drm_fdinfo(&drm_paths, |usage: &mut IntelGpuProc, (keyword, value)| {
                match keyword {
                    // Memory key is in KiB.
                    "drm-total-vram0" => usage.mem_used += value << 10,
                    _ => usage.add_engine_stat(&XE_ENGINES, keyword, value),
                }
            })
        }
    }
}

fn compute_util(
    driver: IntelDriver, prev: &IntelGpuProc, cur: &IntelGpuProc, interval: &Duration,
) -> u32 {
    match driver {
        // Busy-time deltas over the interval, summed across engines (as AMD
        // does).
        IntelDriver::I915 => {
            let util: u64 = prev
                .busy
                .iter()
                .zip(&cur.busy)
                .map(|(&prev, &cur)| diff_usage(prev, cur, interval))
                .sum();

            util.try_into().unwrap_or(0)
        }
        // Ratio of busy cycles to elapsed cycles over the interval.
        IntelDriver::Xe => {
            let prev_total: u64 = prev.total_cycles.iter().sum();
            let cycles_delta = cur
                .busy
                .iter()
                .sum::<u64>()
                .saturating_sub(prev.busy.iter().sum());
            let total_delta = cur
                .total_cycles
                .iter()
                .sum::<u64>()
                .saturating_sub(prev_total);

            if prev_total == 0 || total_delta == 0 {
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
    let mut overall_util_vec = Vec::with_capacity(num_gpus);
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

        if (widgets_to_harvest.use_proc || widgets_to_harvest.use_gpu_graph)
            && let Some(procs) = get_intel_fdinfo(&device_path, driver)
        {
            PREV_PROC_DATA.with_borrow_mut(|prev_proc_data| {
                let has_prev_sample = prev_proc_data.contains_key(&device_path);
                let prev_fdinfo = prev_proc_data.entry(device_path).or_default();
                let mut seen_pids = IntHashSet::default();
                let mut engine_busy = [0.0; NUM_ENGINE_CLASSES];
                let mut engine_capacity = [1; NUM_ENGINE_CLASSES];

                let mut procs_map = IntHashMap::default();
                for (proc_pid, proc_usage) in procs {
                    seen_pids.insert(proc_pid);

                    for (capacity, &reported) in
                        engine_capacity.iter_mut().zip(&proc_usage.capacity)
                    {
                        *capacity = (*capacity).max(reported);
                    }

                    if let Some(prev_usage) = prev_fdinfo.get_mut(&proc_pid) {
                        let gpu_util = compute_util(driver, prev_usage, &proc_usage, &interval);

                        if gpu_util > 0 || proc_usage.mem_used > 0 {
                            procs_map.insert(proc_pid, (proc_usage.mem_used, gpu_util));
                        }

                        let fractions =
                            engine_busy_fractions(driver, prev_usage, &proc_usage, &interval);
                        for (busy, fraction) in engine_busy.iter_mut().zip(fractions) {
                            *busy += fraction;
                        }

                        *prev_usage = proc_usage;
                    } else {
                        prev_fdinfo.insert(proc_pid, proc_usage);
                    }
                }

                prev_fdinfo.retain(|k, _| seen_pids.contains(k));

                if widgets_to_harvest.use_proc && !procs_map.is_empty() {
                    proc_vec.push(procs_map);
                }

                if widgets_to_harvest.use_gpu_graph && has_prev_sample {
                    overall_util_vec.push((
                        device_name,
                        busiest_engine_util(&engine_busy, &engine_capacity),
                    ));
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
        overall_utilization: (!overall_util_vec.is_empty()).then_some(overall_util_vec),
    })
}

#[cfg(test)]
mod tests {
    use std::time::Duration;

    use super::{IntelDriver, IntelGpuProc, engine_busy_fractions};

    fn render_busy(ns: u64) -> IntelGpuProc {
        let mut proc = IntelGpuProc::default();
        proc.busy[0] = ns;
        proc
    }

    #[test]
    fn i915_engine_busy_fractions() {
        let interval = Duration::from_secs(1);

        let fractions = engine_busy_fractions(
            IntelDriver::I915,
            &render_busy(1_000_000),
            &render_busy(1_001_000_000),
            &interval,
        );
        assert_eq!(fractions, [1.0, 0.0, 0.0, 0.0, 0.0]);

        let fractions = engine_busy_fractions(
            IntelDriver::I915,
            &render_busy(500_000_000),
            &render_busy(750_000_000),
            &interval,
        );
        assert_eq!(fractions, [0.25, 0.0, 0.0, 0.0, 0.0]);

        let fractions = engine_busy_fractions(
            IntelDriver::I915,
            &render_busy(900_000_000),
            &render_busy(100_000_000),
            &interval,
        );
        assert_eq!(fractions, [0.0; 5]);
    }
}
