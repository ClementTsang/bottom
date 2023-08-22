use std::mem::{size_of, zeroed};

use windows::Win32::System::ProcessStatus::{GetPerformanceInfo, PERFORMANCE_INFORMATION};

use crate::data_harvester::memory::MemHarvest;

// TODO: Note this actually calculates the total *committed* usage. Rename and change label for accuracy!
/// Get the committed memory usage.
///
/// Code based on [sysinfo's](https://github.com/GuillaumeGomez/sysinfo/blob/6f8178495adcf3ca4696a9ec548586cf6a621bc8/src/windows/system.rs#L169).
pub(crate) fn get_swap_usage() -> Option<MemHarvest> {
    // SAFETY: The safety invariant is that we only touch what's in `perf_info` if it succeeds, and that
    // the bindings are "safe" to use with how we call them.
    unsafe {
        let mut perf_info: PERFORMANCE_INFORMATION = zeroed();
        if GetPerformanceInfo(&mut perf_info, size_of::<PERFORMANCE_INFORMATION>() as u32).is_ok() {
            // Saturating sub by perf_info.PhysicalTotal for what sysinfo does.
            let swap_total = perf_info.PageSize.saturating_mul(perf_info.CommitLimit) as u64;
            let swap_used = perf_info.PageSize.saturating_mul(perf_info.CommitTotal) as u64;

            Some(MemHarvest {
                used_bytes: swap_used,
                total_bytes: swap_total,
                use_percent: Some(swap_used as f64 / swap_total as f64 * 100.0),
            })
        } else {
            None
        }
    }
}
