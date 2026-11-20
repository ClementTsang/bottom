//! Non-Linux GPU data collection via sysinfo.
//!
//! This module provides GPU data on Windows, macOS, and other platforms where
//! sysinfo has built-in GPU support.

use std::num::NonZeroU64;

use sysinfo::Gpus;

use crate::{
    app::layout_manager::UsedWidgets,
    collection::{GpuKey, PciAddress, memory::MemData},
};

pub struct SysinfoGpuData {
    pub memory: Option<Vec<(GpuKey, MemData)>>,
    pub overall_utilization: Option<Vec<(GpuKey, f32)>>,
}

/// Returns the human-readable name for a GPU vendor.
fn vendor_name(vendor: Option<&str>) -> String {
    match vendor {
        Some("Intel Corporation") => "Intel".to_string(),
        Some(vendor) if vendor.starts_with("Advanced Micro Devices") => "AMD".to_string(),
        Some("NVIDIA Corporation") => "NVIDIA".to_string(),
        Some("Apple Inc.") => "Apple".to_string(),
        Some("") | None => "Unknown".to_string(),
        Some(other) => other.to_string(),
    }
}

/// Collect GPU data using sysinfo's `Gpus` type.
///
/// Only includes memory data if the platform reports VRAM (Windows).
/// On macOS/FreeBSD, memory is always `None` because GPUs share
/// system memory with no dedicated VRAM.
///
/// If `skip_nvidia` is set, NVIDIA GPUs are skipped, e.g. if NVML already
/// reported them.
pub fn collect_sysinfo_gpu_data(
    gpus: &Gpus, widgets: &UsedWidgets, skip_nvidia: bool,
) -> Option<SysinfoGpuData> {
    let mut mem_vec: Vec<(GpuKey, MemData)> = Vec::new();
    let mut overall_util_vec: Vec<(GpuKey, f32)> = Vec::new();

    for gpu in gpus.iter() {
        if skip_nvidia && gpu.vendor() == Some("NVIDIA Corporation") {
            continue;
        }

        let name = vendor_name(gpu.vendor());
        let model = gpu.model().unwrap_or("");
        let display_name = match (model.is_empty(), model.starts_with(&name)) {
            (true, _) => name,
            (false, true) => model.to_string(),
            (false, false) => format!("{name} {model}"),
        };

        let pci = gpu.pci();
        let key = GpuKey {
            address: PciAddress {
                domain: pci.domain,
                bus: pci.bus,
                device: pci.device,
                function: pci.function,
            },
            name: display_name,
        };

        if widgets.use_mem {
            // total_memory/used_memory returns Some only on Windows (DXGI).
            // On macOS/FreeBSD these always return None (shared memory).
            if let (Some(total), Some(used)) = (gpu.total_memory(), gpu.used_memory())
                && let Some(total_bytes) = NonZeroU64::new(total)
            {
                mem_vec.push((
                    key.clone(),
                    MemData {
                        total_bytes,
                        used_bytes: used,
                    },
                ));
            }
        }

        if let Some(usage) = gpu.usage() {
            overall_util_vec.push((key, usage.clamp(0.0, 100.0)));
        }
    }

    Some(SysinfoGpuData {
        memory: if !mem_vec.is_empty() {
            Some(mem_vec)
        } else {
            None
        },
        overall_utilization: if !overall_util_vec.is_empty() {
            Some(overall_util_vec)
        } else {
            None
        },
    })
}
