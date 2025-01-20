use std::mem::{size_of, zeroed};

use windows::Win32::System::ProcessStatus::{GetPerformanceInfo, PERFORMANCE_INFORMATION};

use crate::collection::memory::MemHarvest;

const PERFORMANCE_INFORMATION_SIZE: u32 = size_of::<PERFORMANCE_INFORMATION>() as _;

/// Get the committed memory usage.
///
/// Code based on [sysinfo's](https://github.com/GuillaumeGomez/sysinfo/blob/6f8178495adcf3ca4696a9ec548586cf6a621bc8/src/windows/system.rs#L169).
pub(crate) fn get_committed_usage() -> Option<MemHarvest> {
    // SAFETY: The safety invariant is that we only touch what's in `perf_info` if it succeeds, and that
    // the bindings are "safe" to use with how we call them.
    unsafe {
        let mut perf_info: PERFORMANCE_INFORMATION = zeroed();
        if GetPerformanceInfo(&mut perf_info, PERFORMANCE_INFORMATION_SIZE).is_ok() {
            let page_size = perf_info.PageSize;

            let committed_total = page_size.saturating_mul(perf_info.CommitLimit) as u64;
            let committed_used = page_size.saturating_mul(perf_info.CommitTotal) as u64;

            Some(MemHarvest {
                used_bytes: committed_used,
                total_bytes: committed_total,
                use_percent: Some(committed_used as f64 / committed_total as f64 * 100.0),
            })
        } else {
            None
        }
    }
}
