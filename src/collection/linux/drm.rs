//! Shared helpers for collecting GPU data from the Linux DRM subsystem. Primarily used for AMD (`amdgpu`)
//! and Intel (`i915`/`xe`) GPU collectors to gather info via sysfs and read info under `/proc/<pid>/fdinfo/`.
//!
//! See <https://docs.kernel.org/gpu/drm-usage-stats.html> for more info.

use std::{
    fs::{self, read_to_string},
    ops::Mul,
    path::{Path, PathBuf},
    time::Duration,
};

use concat_string::concat_string;
use rustc_hash::FxHashSet as HashSet;

use crate::{
    collection::{linux::utils::is_device_awake, processes::Pid},
    utils::int_hash::IntHashMap,
};

/// Enumerate the PCI device directories bound to a given DRM driver module (e.g. `amdgpu`,
/// `i915`, `xe`).
///
/// Reads `/sys/module/<driver>/drivers/pci:<driver>`, keeping only entries that are GPUs (i.e. have
/// a `drm/` subdirectory) and that are currently awake, so we don't wake a sleeping device.
pub(crate) fn enumerate_drm_devices(driver: &str) -> Option<Vec<PathBuf>> {
    let mut devices = Vec::new();

    // read all PCI devices controlled by the given driver module
    let Ok(paths) = fs::read_dir(concat_string!(
        "/sys/module/",
        driver,
        "/drivers/pci:",
        driver
    )) else {
        return None;
    };

    for path in paths {
        let Ok(path) = path else { continue };

        let device_path = path.path();
        if !device_path.is_dir() {
            continue;
        }

        // Skip if asleep to avoid wakeups.
        if !is_device_awake(&device_path) {
            continue;
        }

        // This will exist for GPUs but not others, this is how we find their kernel name.
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

/// Return the DRM device nodes (e.g. `/dev/dri/renderD128`, `/dev/dri/card0`) for a PCI device, by
/// reading its `drm/` subdirectory.
pub(crate) fn get_drm_render_nodes(device_path: &Path) -> Option<Vec<PathBuf>> {
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

        drm_devices.push(PathBuf::from(concat_string!("/dev/dri/", drm_name)));
    }

    if drm_devices.is_empty() {
        None
    } else {
        Some(drm_devices)
    }
}

/// from amdgpu_top: <https://github.com/Umio-Yasuno/amdgpu_top/blob/c961cf6625c4b6d63fda7f03348323048563c584/crates/libamdgpu_top/src/stat/fdinfo/proc_info.rs#L13-L27>
fn get_pid_fds(pid: Pid, device_paths: &[PathBuf]) -> Option<Vec<u32>> {
    let Ok(fd_list) = fs::read_dir(format!("/proc/{pid}/fd/")) else {
        return None;
    };

    let valid_fds: Vec<u32> = fd_list
        .filter_map(|fd_link| {
            let dir_entry = fd_link.map(|fd_link| fd_link.path()).ok()?;
            let link = fs::read_link(&dir_entry).ok()?;

            // e.g. "/dev/dri/renderD128" or "/dev/dri/card0"
            if device_paths.iter().any(|path| link.starts_with(path)) {
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

// from amdgpu_top: https://github.com/Umio-Yasuno/amdgpu_top/blob/c961cf6625c4b6d63fda7f03348323048563c584/crates/libamdgpu_top/src/stat/fdinfo/proc_info.rs#L114
pub(crate) fn diff_usage(pre: u64, cur: u64, interval: &Duration) -> u64 {
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

/// Scan every process for open fds pointing at the given DRM device nodes, parse each fd's
/// `fdinfo`, and accumulate per-process usage keyed by pid.
///
/// - `T` is the accumulator type (e.g. a struct holding a bunch of counters).
/// - `F` is a function that takes an accumulator and a keyword/value pair from the fdinfo,
///   and updates the accumulator with that info.
pub(crate) fn collect_drm_fdinfo<T, F>(
    render_nodes: &[PathBuf], accumulate: F,
) -> Option<IntHashMap<Pid, T>>
where
    T: Default + PartialEq,
    F: Fn(&mut T, (&str, u64)),
{
    let mut fdinfo = IntHashMap::default();

    let Ok(proc_dir) = fs::read_dir("/proc") else {
        return None;
    };

    let pids: Vec<Pid> = proc_dir
        .filter_map(|dir_entry| {
            // check if pid is valid
            let dir_entry = dir_entry.ok()?;
            let metadata = dir_entry.metadata().ok()?;

            if !metadata.is_dir() {
                return None;
            }

            let pid = dir_entry.file_name().to_str()?.parse::<Pid>().ok()?;

            // skip init process
            if pid == 1 {
                return None;
            }

            Some(pid)
        })
        .collect();

    for pid in pids {
        // collect file descriptors that point to our device renderers
        let Some(fds) = get_pid_fds(pid, render_nodes) else {
            continue;
        };

        let mut current_usage: T = Default::default();

        let mut observed_ids: HashSet<usize> = HashSet::default();

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

                let current_fdinfo = (fdinfo_keyword, fdinfo_value_num);

                accumulate(&mut current_usage, current_fdinfo);
            }
        }

        if current_usage != Default::default() {
            fdinfo.insert(pid, current_usage);
        }
    }

    Some(fdinfo)
}
