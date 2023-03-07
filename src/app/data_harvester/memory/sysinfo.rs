//! Collecting memory data using sysinfo.

use sysinfo::{System, SystemExt};

use crate::data_harvester::memory::MemHarvest;

/// Returns RAM usage.
pub(crate) fn get_ram_usage(sys: &System) -> Option<MemHarvest> {
    let mem_used_in_kib = sys.used_memory() / 1024;
    let mem_total_in_kib = sys.total_memory() / 1024;

    Some(MemHarvest {
        total_kib: mem_total_in_kib,
        used_kib: mem_used_in_kib,
        use_percent: if mem_total_in_kib == 0 {
            None
        } else {
            Some(mem_used_in_kib as f64 / mem_total_in_kib as f64 * 100.0)
        },
    })
}

/// Returns SWAP usage.
#[cfg(not(target_os = "windows"))]
pub(crate) fn get_swap_usage(sys: &System) -> Option<MemHarvest> {
    let mem_used_in_kib = sys.used_swap() / 1024;
    let mem_total_in_kib = sys.total_swap() / 1024;

    Some(MemHarvest {
        total_kib: mem_total_in_kib,
        used_kib: mem_used_in_kib,
        use_percent: if mem_total_in_kib == 0 {
            None
        } else {
            Some(mem_used_in_kib as f64 / mem_total_in_kib as f64 * 100.0)
        },
    })
}
