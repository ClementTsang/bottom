//! Collecting memory data using sysinfo.

use sysinfo::System;

use crate::new_data_collection::sources::memory::MemData;

/// Returns RAM usage.
pub(crate) fn get_ram_usage(sys: &System) -> MemData {
    let mem_used = sys.used_memory();
    let mem_total = sys.total_memory();

    MemData {
        used_bytes: mem_used,
        total_bytes: mem_total,
    }
}

/// Returns SWAP usage.
pub(crate) fn get_swap_usage(sys: &System) -> MemData {
    let mem_used = sys.used_swap();
    let mem_total = sys.total_swap();

    MemData {
        used_bytes: mem_used,
        total_bytes: mem_total,
    }
}

/// Returns cache usage. sysinfo has no way to do this directly but it should
/// equal the difference between the available and free memory. Free memory is
/// defined as memory not containing any data, which means cache and buffer
/// memory are not "free". Available memory is defined as memory able
/// to be allocated by processes, which includes cache and buffer memory. On
/// Windows, this will always be 0 - as such, we do not use this on Windows.
///
/// For more information, see [docs](https://docs.rs/sysinfo/latest/sysinfo/struct.System.html#method.available_memory)
/// and [memory explanation](https://askubuntu.com/questions/867068/what-is-available-memory-while-using-free-command)
#[cfg(not(target_os = "windows"))]
pub(crate) fn get_cache_usage(sys: &System) -> MemData {
    let mem_used = sys.available_memory().saturating_sub(sys.free_memory());
    let mem_total = sys.total_memory();

    MemData {
        total_bytes: mem_total,
        used_bytes: mem_used,
    }
}
