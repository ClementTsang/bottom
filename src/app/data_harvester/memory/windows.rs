use std::mem::{size_of, zeroed};
use windows::Win32::Foundation::TRUE;
use windows::Win32::System::ProcessStatus::{K32GetPerformanceInfo, PERFORMANCE_INFORMATION};

use crate::data_harvester::memory::MemHarvest;

// TODO: Note this actually calculates the total *committed* usage. Rename and change label for accuracy!
pub(crate) fn get_swap_usage() -> Option<MemHarvest> {
    unsafe {
        let mut perf_info: PERFORMANCE_INFORMATION = zeroed();
        if K32GetPerformanceInfo(&mut perf_info, size_of::<PERFORMANCE_INFORMATION>() as u32)
            == TRUE
        {
            let swap_total = perf_info.PageSize.saturating_mul(perf_info.CommitLimit) as u64;

            let swap_used = perf_info.PageSize.saturating_mul(perf_info.CommitTotal) as u64;

            Some(MemHarvest {
                total_kib: swap_total / 1024,
                used_kib: swap_used / 1024,
                use_percent: Some(swap_used as f64 / swap_total as f64 * 100.0),
            })
        } else {
            None
        }
    }
}
