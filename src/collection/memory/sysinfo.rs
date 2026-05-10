//! Collecting memory data using sysinfo.

use std::num::NonZeroU64;

use sysinfo::System;

use crate::collection::memory::MemData;

#[inline]
fn get_usage(used: u64, total: u64) -> Option<MemData> {
    NonZeroU64::new(total).map(|total_bytes| MemData {
        total_bytes,
        used_bytes: used,
    })
}

/// Returns RAM usage.
pub(crate) fn get_ram_usage(sys: &System) -> Option<MemData> {
    cfg_if::cfg_if! {
        if #[cfg(target_os = "linux")] {
            use crate::collection::linux::cgroups;

            let base_used = sys.used_memory();
            let base_total = sys.total_memory();

            let cgroup_data = cgroups::get_cgroup_memory_data();

            let (used, total) = match cgroup_data {
                Some(cgroup_data) => {
                    let used = cgroup_data.used_bytes;
                    let total = match cgroup_data.limit {
                        Some(cgroups::CgroupMemLimit::Bytes(bytes)) => bytes,
                        Some(cgroups::CgroupMemLimit::Max) => base_total,
                        None => base_total,
                    };

                    (used, total)
                }
                None => (base_used, base_total),
            };

            get_usage(used, total)
        } else {
            get_usage(sys.used_memory(), sys.total_memory())
        }
    }
}

/// Returns SWAP usage.
#[cfg(not(target_os = "windows"))]
pub(crate) fn get_swap_usage(sys: &System) -> Option<MemData> {
    get_usage(sys.used_swap(), sys.total_swap())
}

/// Returns cache usage. sysinfo has no way to do this directly but it should
/// equal the difference between the available and free memory. Free memory is
/// defined as memory not containing any data, which means cache and buffer
/// memory are not "free". Available memory is defined as memory able
/// to be allocated by processes, which includes cache and buffer memory. On
/// Windows, this will always be 0. For more information, see [docs](https://docs.rs/sysinfo/latest/sysinfo/struct.System.html#method.available_memory)
/// and [memory explanation](https://askubuntu.com/questions/867068/what-is-available-memory-while-using-free-command)
#[cfg(not(target_os = "windows"))]
pub(crate) fn get_cache_usage(sys: &System) -> Option<MemData> {
    let mem_used = sys.available_memory().saturating_sub(sys.free_memory());
    let mem_total = sys.total_memory();

    get_usage(mem_used, mem_total)
}
