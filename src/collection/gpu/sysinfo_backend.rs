//! Non-Linux GPU data collection via sysinfo.
//!
//! This module provides GPU data on Windows, macOS, and other platforms where
//! sysinfo has built-in GPU support.

use std::num::NonZeroU64;

use sysinfo::Gpus;

use crate::{app::layout_manager::UsedWidgets, collection::memory::MemData};

/// Returns the human-readable name for a GPU vendor.
fn vendor_name(vendor: Option<&str>) -> String {
    match vendor {
        Some("Intel Corporation") => "Intel".to_string(),
        Some("Advanced Micro Devices, Inc.") => "AMD".to_string(),
        Some("NVIDIA Corporation") => "NVIDIA".to_string(),
        Some("") | None => "Unknown".to_string(),
        Some(other) => other.to_string(),
    }
}

/// Collect GPU data using sysinfo's `Gpus` type.
///
/// Only includes memory data if the platform reports VRAM (Windows).
/// On macOS/FreeBSD, memory is always `None` because GPUs share
/// system memory with no dedicated VRAM.
pub fn collect_sysinfo_gpu_data(
    gpus: &Gpus, widgets: &UsedWidgets,
) -> Option<Vec<(String, MemData)>> {
    let mut mem_vec: Vec<(String, MemData)> = Vec::new();

    for gpu in gpus.iter() {
        let name = vendor_name(gpu.vendor());
        let model = gpu.model().unwrap_or("");
        let display_name = match (model.is_empty(), model.starts_with(&name)) {
            (true, _) | (false, true) => name,
            (false, false) => format!("{name} {model}"),
        };

        if widgets.use_mem {
            // total_memory/used_memory returns Some only on Windows (DXGI).
            // On macOS/FreeBSD these always return None (shared memory).
            if let (Some(total), Some(used)) = (gpu.total_memory(), gpu.used_memory())
                && let Some(total_bytes) = NonZeroU64::new(total)
            {
                mem_vec.push((
                    display_name,
                    MemData {
                        total_bytes,
                        used_bytes: used,
                    },
                ));
            }
        }
    }

    if !mem_vec.is_empty() {
        Some(mem_vec)
    } else {
        None
    }
}
