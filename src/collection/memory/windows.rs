use std::{mem::zeroed, num::NonZeroU64};

use sysinfo::System;
use windows::{
    Win32::{
        Foundation::ERROR_SUCCESS,
        System::Performance::{
            PDH_FMT_COUNTERVALUE, PDH_FMT_DOUBLE, PDH_HCOUNTER, PDH_HQUERY, PdhAddEnglishCounterW,
            PdhCloseQuery, PdhCollectQueryData, PdhGetFormattedCounterValue, PdhOpenQueryW,
            PdhRemoveCounter,
        },
    },
    core::w,
};

use crate::collection::memory::MemData;

/// Get swap memory usage on Windows. This does it by using checking Windows' performance counters.
/// This is based on the technique done by psutil [here](https://github.com/giampaolo/psutil/pull/2160).
///
/// Also see:
/// - <https://github.com/GuillaumeGomez/sysinfo/blob/master/src/windows/system.rs>
/// - <https://learn.microsoft.com/en-us/windows/win32/api/psapi/ns-psapi-performance_information>
/// - <https://en.wikipedia.org/wiki/Commit_charge>.
/// - <https://github.com/giampaolo/psutil/issues/2431>
/// - <https://github.com/oshi/oshi/issues/1175>
/// - <https://github.com/oshi/oshi/issues/1182>
pub(crate) fn get_swap_usage(sys: &System) -> Option<MemData> {
    let total_bytes = NonZeroU64::new(sys.total_swap())?;

    // See https://kennykerr.ca/rust-getting-started/string-tutorial.html
    let query = w!("\\Paging File(_Total)\\% Usage");

    // SAFETY: Hits a few Windows APIs; this should be safe as we check each step, and
    // we clean up at the end.
    unsafe {
        let mut query_handle: PDH_HQUERY = zeroed();
        let mut counter_handle: PDH_HCOUNTER = zeroed();
        let mut counter_value: PDH_FMT_COUNTERVALUE = zeroed();

        if PdhOpenQueryW(None, 0, &mut query_handle) != ERROR_SUCCESS.0 {
            return None;
        }

        if PdhAddEnglishCounterW(query_handle, query, 0, &mut counter_handle) != ERROR_SUCCESS.0 {
            return None;
        }

        // May fail if swap is disabled.
        if PdhCollectQueryData(query_handle) != ERROR_SUCCESS.0 {
            return None;
        }

        if PdhGetFormattedCounterValue(counter_handle, PDH_FMT_DOUBLE, None, &mut counter_value)
            != ERROR_SUCCESS.0
        {
            // If we fail, still clean up.
            PdhCloseQuery(query_handle);
            return None;
        }

        let use_percentage = counter_value.Anonymous.doubleValue;

        // Cleanup.
        PdhRemoveCounter(counter_handle);
        PdhCloseQuery(query_handle);

        let used_bytes = (total_bytes.get() as f64 / 100.0 * use_percentage) as u64;
        Some(MemData {
            used_bytes,
            total_bytes,
        })
    }
}

#[cfg(all(target_os = "windows", test))]
mod tests {
    use sysinfo::{MemoryRefreshKind, RefreshKind};

    use super::*;

    #[test]
    fn test_windows_get_swap_usage() {
        let sys = System::new_with_specifics(
            RefreshKind::nothing().with_memory(MemoryRefreshKind::nothing().with_swap()),
        );

        let swap_usage = get_swap_usage(&sys);
        if sys.total_swap() > 0 {
            // Not sure if we can guarantee this to always pass on a machine, so I'll just print out.
            println!("swap: {swap_usage:?}");
        } else {
            println!("No swap, skipping.");
        }
    }
}
