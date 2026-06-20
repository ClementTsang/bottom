//! Collecting memory data using sysinfo.

use std::num::NonZeroU64;

use crate::collection::{DataCollector, memory::MemData};

#[cfg(target_os = "linux")]
use crate::collection::linux::cgroups::CgroupMemLimit;

#[inline]
fn get_usage(used: u64, total: u64) -> Option<MemData> {
    NonZeroU64::new(total).map(|total_bytes| MemData {
        total_bytes,
        used_bytes: used,
    })
}

/// Resolves the total memory to report given an optional cgroup limit and the physical total.
///
/// cgroup v1 reports an "unlimited" limit as a very large value, which causes problems if taken
/// literally. This function caps it to the minimum of the cgroup total or the actual total to
/// avoid this problem.
#[cfg(target_os = "linux")]
#[inline]
fn resolve_cgroup_total(limit: Option<&CgroupMemLimit>, base_total: u64) -> u64 {
    match limit {
        Some(CgroupMemLimit::Bytes(bytes)) => (*bytes).min(base_total),
        Some(CgroupMemLimit::Max) | None => base_total,
    }
}

/// Returns memory (RAM) usage using sysinfo.
///
/// On Linux, this will take cgroup usage/limits into account.
pub(crate) fn get_ram_usage(collector: &DataCollector) -> Option<MemData> {
    let sys = &collector.sys.system;

    cfg_select! {
        target_os = "linux" => {
            let base_used = sys.used_memory();
            let base_total = sys.total_memory();

            let (used, total) = match &collector.cgroup_memory_data.ram {
                Some(cgroup_data) => {
                    let used = cgroup_data.used_bytes;
                    let total = resolve_cgroup_total(cgroup_data.limit.as_ref(), base_total);

                    (used, total)
                }
                None => (base_used, base_total),
            };

            get_usage(used, total)
        }
        _ => {
            get_usage(sys.used_memory(), sys.total_memory())
        }
    }
}

/// Returns SWAP usage using sysinfo.
///
/// On Linux, this will take cgroup usage/limits into account.
#[cfg(not(target_os = "windows"))]
pub(crate) fn get_swap_usage(collector: &DataCollector) -> Option<MemData> {
    let sys = &collector.sys.system;

    cfg_select! {
        target_os = "linux" => {
            let base_used = sys.used_swap();
            let base_total = sys.total_swap();

            let (used, total) = match &collector.cgroup_memory_data.swap {
                Some(cgroup_data) => {
                    let used = cgroup_data.used_bytes;
                    let total = resolve_cgroup_total(cgroup_data.limit.as_ref(), base_total);

                    (used, total)
                }
                None => (base_used, base_total),
            };

            get_usage(used, total)
        }
        _ => {
            get_usage(sys.used_swap(), sys.total_swap())
        }
    }
}

/// Returns cache usage using sysinfo.
///
/// sysinfo has no way to do this directly but it should equal the difference
/// between the available and free memory. Free memory is defined as memory
/// not containing any data, which means cache and buffer memory are not
/// "free". Available memory is defined as memory able to be allocated by
/// processes, which includes cache and buffer memory. On Windows, this will
/// always be 0.
///
/// For more information, see [sysinfo docs](https://docs.rs/sysinfo/latest/sysinfo/struct.System.html#method.available_memory)
/// and [this explanation on memory](https://askubuntu.com/questions/867068/what-is-available-memory-while-using-free-command)
#[cfg(not(target_os = "windows"))]
pub(crate) fn get_cache_usage(sys: &sysinfo::System) -> Option<MemData> {
    let mem_used = sys.available_memory().saturating_sub(sys.free_memory());
    let mem_total = sys.total_memory();

    get_usage(mem_used, mem_total)
}

#[cfg(test)]
#[cfg(target_os = "linux")]
mod linux_tests {
    use super::*;

    const BASE_TOTAL: u64 = 16 * 1024 * 1024 * 1024; // 16 GiB

    /// Regression test for <https://github.com/ClementTsang/bottom/issues/2092>.
    #[test]
    fn cap_cgroup_v1_limits() {
        let sentinel = u64::MAX;
        assert_eq!(
            resolve_cgroup_total(Some(&CgroupMemLimit::Bytes(sentinel)), BASE_TOTAL),
            BASE_TOTAL
        );
    }

    #[test]
    fn legit_cgroup_limit_works() {
        let limit = 4 * 1024 * 1024 * 1024; // 4 GiB
        assert_eq!(
            resolve_cgroup_total(Some(&CgroupMemLimit::Bytes(limit)), BASE_TOTAL),
            limit
        );
    }

    #[test]
    fn max_and_missing_limit_use_physical_total() {
        assert_eq!(
            resolve_cgroup_total(Some(&CgroupMemLimit::Max), BASE_TOTAL),
            BASE_TOTAL
        );
        assert_eq!(resolve_cgroup_total(None, BASE_TOTAL), BASE_TOTAL);
    }
}
