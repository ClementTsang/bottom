use std::{
    mem::{size_of, zeroed},
    num::NonZeroU64,
};

use windows::Win32::System::ProcessStatus::{GetPerformanceInfo, PERFORMANCE_INFORMATION};

use crate::collection::memory::MemData;

const PERFORMANCE_INFORMATION_SIZE: u32 = size_of::<PERFORMANCE_INFORMATION>() as _;

/// Get the committed memory usage.
///
/// Code based on [sysinfo's](https://github.com/GuillaumeGomez/sysinfo/blob/6f8178495adcf3ca4696a9ec548586cf6a621bc8/src/windows/system.rs#L169).
pub(crate) fn get_committed_usage() -> Option<MemData> {
    // SAFETY: The safety invariant is that we only touch what's in `perf_info` if it succeeds, and that
    // the bindings are "safe" to use with how we call them.
    unsafe {
        let mut perf_info: PERFORMANCE_INFORMATION = zeroed();
        if GetPerformanceInfo(&mut perf_info, PERFORMANCE_INFORMATION_SIZE).is_ok() {
            let page_size = perf_info.PageSize;

            let Some(committed_total) =
                NonZeroU64::new(page_size.saturating_mul(perf_info.CommitLimit) as u64)
            else {
                return None;
            };
            let committed_used = page_size.saturating_mul(perf_info.CommitTotal) as u64;

            Some(MemData {
                used_bytes: committed_used,
                total_bytes: committed_total,
            })
        } else {
            None
        }
    }
}
