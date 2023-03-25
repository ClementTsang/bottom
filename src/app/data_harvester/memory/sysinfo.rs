//! Collecting memory data using sysinfo.

use sysinfo::{System, SystemExt};

use crate::data_harvester::memory::MemHarvest;

/// Returns RAM usage.
pub(crate) fn get_ram_usage(sys: &System) -> Option<MemHarvest> {
    let mem_used = sys.used_memory();
    let mem_total = sys.total_memory();

    Some(MemHarvest {
        used_bytes: mem_used,
        total_bytes: mem_total,
        use_percent: if mem_total == 0 {
            None
        } else {
            Some(mem_used as f64 / mem_total as f64 * 100.0)
        },
    })
}

/// Returns SWAP usage.
#[cfg(not(target_os = "windows"))]
pub(crate) fn get_swap_usage(sys: &System) -> Option<MemHarvest> {
    let mem_used = sys.used_swap();
    let mem_total = sys.total_swap();

    Some(MemHarvest {
        used_bytes: mem_used,
        total_bytes: mem_total,
        use_percent: if mem_total == 0 {
            None
        } else {
            Some(mem_used as f64 / mem_total as f64 * 100.0)
        },
    })
}
