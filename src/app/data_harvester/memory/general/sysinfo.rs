//! Data collection for memory via sysinfo.

use crate::data_harvester::memory::MemHarvest;
use sysinfo::{System, SystemExt};

pub async fn get_mem_data(
    sys: &System, actually_get: bool,
) -> (
    crate::utils::error::Result<Option<MemHarvest>>,
    crate::utils::error::Result<Option<MemHarvest>>,
) {
    use futures::join;

    if !actually_get {
        (Ok(None), Ok(None))
    } else {
        join!(get_ram_data(sys), get_swap_data(sys))
    }
}

pub async fn get_ram_data(sys: &System) -> crate::utils::error::Result<Option<MemHarvest>> {
    let (mem_total_in_kib, mem_used_in_kib) = (sys.total_memory(), sys.used_memory());

    Ok(Some(MemHarvest {
        mem_total_in_kib,
        mem_used_in_kib,
        use_percent: if mem_total_in_kib == 0 {
            None
        } else {
            Some(mem_used_in_kib as f64 / mem_total_in_kib as f64 * 100.0)
        },
    }))
}

pub async fn get_swap_data(sys: &System) -> crate::utils::error::Result<Option<MemHarvest>> {
    let (mem_total_in_kib, mem_used_in_kib) = (sys.total_swap(), sys.used_swap());

    Ok(Some(MemHarvest {
        mem_total_in_kib,
        mem_used_in_kib,
        use_percent: if mem_total_in_kib == 0 {
            None
        } else {
            Some(mem_used_in_kib as f64 / mem_total_in_kib as f64 * 100.0)
        },
    }))
}
