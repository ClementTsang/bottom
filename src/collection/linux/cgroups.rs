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
#[derive(Default, Debug)]
pub(crate) struct CgroupCpuCollector {
    /// Effective number of CPUs based on quota/period ratio.
    pub quota_cpus: Option<f64>,
    /// Computed average CPU usage percent (only set when a quota is active).
    pub avg_cpu_percent: Option<f32>,
    prev_cpu_usec: Option<u64>,
    prev_time: Option<std::time::Instant>,
}

impl CgroupCpuCollector {
    pub(crate) fn refresh(&mut self) {
        if !self.try_update_cpu_cgroup_v1() && !self.try_update_cpu_cgroup_v2() {
            self.quota_cpus = None;
            self.avg_cpu_percent = None;
            self.prev_cpu_usec = None;
            self.prev_time = None;
        }
    }

    /// Try to update CPU data using cgroup v1 semantics. Returns `true` on success.
    fn try_update_cpu_cgroup_v1(&mut self) -> bool {
        let quota_str = match fs::read_to_string("/sys/fs/cgroup/cpu/cpu.cfs_quota_us") {
            Ok(s) => s,
            Err(_) => return false,
        };
        let quota_raw: i64 = match quota_str.trim().parse() {
            Ok(v) => v,
            Err(_) => return false,
        };

        let period = read_u64("/sys/fs/cgroup/cpu/cpu.cfs_period_us");
        self.quota_cpus = if quota_raw > 0 {
            period.and_then(|p| if p > 0 { Some(quota_raw as f64 / p as f64) } else { None })
        } else {
            None // -1 = unlimited
        };

        // cpuacct.usage is in nanoseconds; convert to microseconds for consistency with v2
        if let Some(usage_nsec) = read_u64("/sys/fs/cgroup/cpuacct/cpuacct.usage") {
            self.compute_avg_and_update(usage_nsec / 1000);
        } else {
            self.avg_cpu_percent = None;
        }

        true
    }

    /// Try to update CPU data using cgroup v2 semantics. Returns `true` on success.
    fn try_update_cpu_cgroup_v2(&mut self) -> bool {
        let cpu_max = match fs::read_to_string("/sys/fs/cgroup/cpu.max") {
            Ok(s) => s,
            Err(_) => return false,
        };

        let mut parts = cpu_max.trim().split_whitespace();
        let quota_str = parts.next().unwrap_or("");
        let period_str = parts.next().unwrap_or("");

        let period: u64 = match period_str.parse() {
            Ok(v) => v,
            Err(_) => return false,
        };

        self.quota_cpus = if quota_str != "max" {
            match quota_str.parse::<u64>() {
                Ok(quota) if quota > 0 && period > 0 => Some(quota as f64 / period as f64),
                _ => None,
            }
        } else {
            None // "max" = unlimited
        };

        // cpu.stat usage_usec is already in microseconds
        if let Some(usage_usec) = read_stat_key("/sys/fs/cgroup/cpu.stat", "usage_usec") {
            self.compute_avg_and_update(usage_usec);
        } else {
            self.avg_cpu_percent = None;
        }

        true
    }

    fn compute_avg_and_update(&mut self, current_usec: u64) {
        let now = std::time::Instant::now();

        self.avg_cpu_percent = if let (Some(quota_cpus), Some(prev_usec), Some(prev_time)) =
            (self.quota_cpus, self.prev_cpu_usec, self.prev_time)
        {
            let elapsed_usec = now.duration_since(prev_time).as_micros() as u64;
            if elapsed_usec > 0 && quota_cpus > 0.0 {
                let delta_usec = current_usec.saturating_sub(prev_usec);
                let pct =
                    (delta_usec as f64 / (elapsed_usec as f64 * quota_cpus) * 100.0) as f32;
                Some(pct.clamp(0.0, 100.0))
            } else {
                None
            }
        } else {
            None
        };

        self.prev_cpu_usec = Some(current_usec);
        self.prev_time = Some(now);
    }
}
