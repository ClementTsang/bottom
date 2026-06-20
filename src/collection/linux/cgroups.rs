//! cgroup-related code for Linux.
//!
//! For info about cgroups, see things like [the kernel docs](https://www.kernel.org/doc/html/latest/admin-guide/cgroup-v2.html)
//! and [Kubernetes docs](https://kubernetes.io/docs/concepts/architecture/cgroups/#deprecation-of-cgroup-v1).

use std::{fs, io::BufRead, time::Instant};

fn read_u64(path: &str) -> Option<u64> {
    fs::read_to_string(path).ok()?.trim().parse().ok()
}

fn read_stat_key(path: &str, key: &str) -> Option<u64> {
    // TODO: Maybe check if this is worth it for the files we read.
    let file = fs::File::open(path).ok()?;
    for line in std::io::BufReader::new(file).lines().map_while(Result::ok) {
        if let Some(rest) = line.strip_prefix(key)
            && let Some(val) = rest.strip_prefix(' ')
        {
            return val.trim().parse().ok();
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

#[inline]
fn parse_cpu_quota(cpu_max: String) -> Option<f64> {
    let mut parts = cpu_max.split_whitespace();
    let quota_str = parts.next().unwrap_or("");

    if quota_str != "max" {
        let period: u64 = {
            let period_str = parts.next().unwrap_or("");

            match period_str.parse() {
                Ok(v) => v,
                Err(_) => return None,
            }
        };

        match quota_str.parse::<u64>() {
            Ok(quota) if quota > 0 && period > 0 => Some(quota as f64 / period as f64),
            _ => None,
        }
    } else {
        None // "max" = unlimited
    }
}

/// Gathers CPU data from cgroup sources.
#[derive(Default, Debug)]
pub(crate) struct CgroupCpuCollector {
    /// A maximum number of CPUs (cores) that can be used, as defined by the cgroup.
    pub cpu_quota: Option<f64>,

    /// Computed average CPU usage percent (only set when a quota is active).
    pub avg_cpu_percent: Option<f32>,

    /// The previous CPU microsecond time (usage) and when it was last updated. Used to compute average cgroup CPU usage.
    prev_cpu: Option<(u64, Instant)>,
}

impl CgroupCpuCollector {
    pub(crate) fn refresh(&mut self) {
        if !self.try_update_cpu_cgroup_v1() && !self.try_update_cpu_cgroup_v2() {
            self.cpu_quota = None;
            self.avg_cpu_percent = None;
            self.prev_cpu = None;
        }
    }

    /// Try to update CPU data using cgroup v1 semantics. Returns `true` on success.
    fn try_update_cpu_cgroup_v1(&mut self) -> bool {
        let quota_raw: i64 = {
            let quota_str = match fs::read_to_string("/sys/fs/cgroup/cpu/cpu.cfs_quota_us") {
                Ok(s) => s,
                Err(_) => return false,
            };

            match quota_str.trim().parse() {
                Ok(v) => v,
                Err(_) => return false,
            }
        };

        self.cpu_quota = if quota_raw > 0 {
            let period = read_u64("/sys/fs/cgroup/cpu/cpu.cfs_period_us");

            period.and_then(|p| {
                if p > 0 {
                    Some(quota_raw as f64 / p as f64)
                } else {
                    None
                }
            })
        } else {
            // If it's less than 0 (-1) then it's representing "unlimited" quota (AKA use the full CPU).
            None
        };

        // cpuacct.usage is in nanoseconds; convert to microseconds for consistency with v2
        if let Some(usage_nsec) = read_u64("/sys/fs/cgroup/cpuacct/cpuacct.usage") {
            self.try_compute_avg_and_update(usage_nsec / 1000);
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

        self.cpu_quota = parse_cpu_quota(cpu_max);

        // cpu.stat usage_usec is in microseconds
        if let Some(current_microseconds) = read_stat_key("/sys/fs/cgroup/cpu.stat", "usage_usec") {
            self.try_compute_avg_and_update(current_microseconds);
        } else {
            self.avg_cpu_percent = None;
        }

        true
    }

    /// Try and compute average CPU usage based on the current microseconds. Note that this requires _two_ invocations
    /// to compute a value, as the first invocation just sets the baseline for the next CPU time and timestamp to
    /// compare with.
    fn try_compute_avg_and_update(&mut self, current_microseconds: u64) {
        let now = Instant::now();

        self.avg_cpu_percent = if let (Some(cpu_quota), Some(prev_cpu)) =
            (self.cpu_quota, self.prev_cpu)
        {
            let (prev_microseconds, prev_time) = prev_cpu;

            let elapsed_microseconds = now.duration_since(prev_time).as_micros() as u64;
            if elapsed_microseconds > 0 && cpu_quota > 0.0 {
                let delta = current_microseconds.saturating_sub(prev_microseconds);
                let pct = (delta as f64 / (elapsed_microseconds as f64 * cpu_quota) * 100.0) as f32;

                Some(pct.clamp(0.0, 100.0))
            } else {
                None
            }
        } else {
            None
        };

        self.prev_cpu = Some((current_microseconds, now));
    }
}
