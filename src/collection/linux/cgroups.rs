//! cgroup-related code for Linux.

use std::{fs, io::BufRead};

/// cgroup memory limits.
pub(crate) enum CgroupMemLimit {
    Bytes(u64),
    Max,
}

/// cgroup memory usage data.
pub(crate) struct CgroupMemData {
    pub used_bytes: u64,
    pub limit: Option<CgroupMemLimit>,
}

fn read_u64(path: &str) -> Option<u64> {
    fs::read_to_string(path).ok()?.trim().parse().ok()
}

fn read_stat_key(path: &str, key: &str) -> Option<u64> {
    let file = fs::File::open(path).ok()?;
    for line in std::io::BufReader::new(file).lines().map_while(Result::ok) {
        if let Some(rest) = line.strip_prefix(key) {
            if let Some(val) = rest.strip_prefix(' ') {
                return val.trim().parse().ok();
            }
        }
    }

    None
}

/// Get cgroup memory data if available.
///
/// Note that we return [`None`] if we couldn't get cgroup memory usage
/// _or_ if there's no cgroups at all, and we do not distinguish between
/// these two cases at the moment.
///
/// Based on [docker's CLI](https://github.com/docker/cli/blob/master/cli/command/container/stats_helpers.go#L254).
pub(crate) fn get_cgroup_memory_data() -> Option<CgroupMemData> {
    // cgroups v1
    if let Some(usage) = read_u64("/sys/fs/cgroup/memory/memory.usage_in_bytes") {
        let inactive = read_stat_key("/sys/fs/cgroup/memory/memory.stat", "total_inactive_file");
        let used_bytes = match inactive {
            Some(inactive) if inactive < usage => usage - inactive,
            _ => usage,
        };

        // Technically if it's like, some insanely high value (https://unix.stackexchange.com/a/421182)
        // then it's "unlimited" but we can just make it so we take the max of the main and this anyway.
        let limit =
            read_u64("/sys/fs/cgroup/memory/memory.limit_in_bytes").map(CgroupMemLimit::Bytes);

        return Some(CgroupMemData { used_bytes, limit });
    }

    // cgroups v2
    if let Some(current) = read_u64("/sys/fs/cgroup/memory.current") {
        let inactive = read_stat_key("/sys/fs/cgroup/memory.stat", "inactive_file");
        let used_bytes = match inactive {
            Some(inactive) if inactive < current => current - inactive,
            _ => current,
        };

        let limit = fs::read_to_string("/sys/fs/cgroup/memory.max")
            .ok()
            .and_then(|s| match s.trim() {
                "max" => Some(CgroupMemLimit::Max),
                v => v.parse::<u64>().map(CgroupMemLimit::Bytes).ok(),
            });

        return Some(CgroupMemData { used_bytes, limit });
    }

    None
}
