//! cgroup-related code for Linux.
//!
//! For info about cgroups, see things like [the kernel docs](https://www.kernel.org/doc/html/latest/admin-guide/cgroup-v2.html)
//! and [Kubernetes docs](https://kubernetes.io/docs/concepts/architecture/cgroups/#deprecation-of-cgroup-v1).

use std::{fs, io::BufRead};

fn read_u64(path: &str) -> Option<u64> {
    fs::read_to_string(path).ok()?.trim().parse().ok()
}

fn read_stat_key(path: &str, key: &str) -> Option<u64> {
    // TODO: Maybe check if this is worth it for the files we read.
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

/// Represents cgroup memory limits.
#[derive(Debug)]
pub(crate) enum CgroupMemLimit {
    Bytes(u64),
    Max,
}

/// Represents cgroup memory usage data.
#[derive(Debug)]
pub(crate) struct CgroupMemData {
    pub used_bytes: u64,
    pub limit: Option<CgroupMemLimit>,
}

/// Gathers memory data from cgroup sources.
#[derive(Default, Debug)]
pub(crate) struct CgroupMemCollector {
    pub ram: Option<CgroupMemData>,
    pub swap: Option<CgroupMemData>,
}

impl CgroupMemCollector {
    /// Refresh the cgroup memory data.
    ///
    /// Based on [docker's CLI](https://github.com/docker/cli/blob/master/cli/command/container/stats_helpers.go#L254).
    pub(crate) fn refresh(&mut self) {
        if !self.try_update_memory_cgroup_v1() && !self.try_update_memory_cgroup_v2() {
            self.ram = None;
            self.swap = None;
        }
    }

    /// Try and update the memory using cgroup v1 semantics. If successful, returns `true`.
    fn try_update_memory_cgroup_v1(&mut self) -> bool {
        if let Some(mem_usage) = read_u64("/sys/fs/cgroup/memory/memory.usage_in_bytes") {
            // --- Memory ---
            let inactive =
                read_stat_key("/sys/fs/cgroup/memory/memory.stat", "total_inactive_file");
            let used_bytes = match inactive {
                Some(inactive) if inactive < mem_usage => mem_usage - inactive,
                _ => mem_usage,
            };

            // Technically if it's like, some insanely high value (https://unix.stackexchange.com/a/421182)
            // then it's "unlimited" but we can just make it so we take the max of the main and this anyway.
            let mem_limit_raw = read_u64("/sys/fs/cgroup/memory/memory.limit_in_bytes");
            let mem_limit = mem_limit_raw.map(CgroupMemLimit::Bytes);

            self.ram = Some(CgroupMemData {
                used_bytes,
                limit: mem_limit,
            });

            // --- Swap ---
            // Since swap is dependent on the normal memory usage, we couple it together.
            if let Some(memsw) = read_u64("/sys/fs/cgroup/memory/memory.memsw.usage_in_bytes") {
                let used_bytes = memsw.saturating_sub(mem_usage);

                // Same idea for here.
                let swap_limit = read_u64("/sys/fs/cgroup/memory/memory.memsw.limit_in_bytes")
                    .map(|memsw_limit| memsw_limit.saturating_sub(mem_limit_raw.unwrap_or(0)))
                    .map(CgroupMemLimit::Bytes);

                self.swap = Some(CgroupMemData {
                    used_bytes,
                    limit: swap_limit,
                });
            }

            true
        } else {
            false
        }
    }

    /// Try and update the memory using cgroup v2 semantics. If successful, returns `true`.
    fn try_update_memory_cgroup_v2(&mut self) -> bool {
        let mut could_update = false;

        if let Some(mem_current) = read_u64("/sys/fs/cgroup/memory.current") {
            // --- Memory ---
            let inactive = read_stat_key("/sys/fs/cgroup/memory.stat", "inactive_file");
            let used_bytes = match inactive {
                Some(inactive) if inactive < mem_current => mem_current - inactive,
                _ => mem_current,
            };

            let limit = fs::read_to_string("/sys/fs/cgroup/memory.max")
                .ok()
                .and_then(|s| match s.trim() {
                    "max" => Some(CgroupMemLimit::Max),
                    v => v.parse::<u64>().map(CgroupMemLimit::Bytes).ok(),
                });

            self.ram = Some(CgroupMemData { used_bytes, limit });

            could_update = true;
        }

        // --- Swap ---
        if let Some(swap_current) = read_u64("/sys/fs/cgroup/memory.swap.current") {
            let limit = fs::read_to_string("/sys/fs/cgroup/memory.swap.max")
                .ok()
                .and_then(|s| match s.trim() {
                    "max" => Some(CgroupMemLimit::Max),
                    v => v.parse::<u64>().map(CgroupMemLimit::Bytes).ok(),
                });

            self.swap = Some(CgroupMemData {
                used_bytes: swap_current,
                limit,
            });

            could_update = true;
        }

        could_update
    }
}

/// Gathers CPU data from cgroup sources.
pub(crate) struct CgroupCpuCollector {}

impl CgroupCpuCollector {
    pub(crate) fn refresh(&mut self) {}
}
