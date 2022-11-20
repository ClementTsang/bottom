//! Process data collection for macOS.  Uses sysinfo and custom bindings.

use sysinfo::System;

use super::ProcessHarvest;
use crate::{data_harvester::processes::UserTable, Pid};
mod sysctl_bindings;

pub fn get_process_data(
    sys: &System, use_current_cpu_total: bool, per_core_percentage: bool, mem_total_kb: u64,
    user_table: &mut UserTable,
) -> crate::utils::error::Result<Vec<ProcessHarvest>> {
    super::macos_freebsd::get_process_data(
        sys,
        use_current_cpu_total,
        per_core_percentage,
        mem_total_kb,
        user_table,
        get_macos_process_cpu_usage,
    )
}

pub(crate) fn fallback_macos_ppid(pid: Pid) -> Option<Pid> {
    sysctl_bindings::kinfo_process(pid)
        .map(|kinfo| kinfo.kp_eproc.e_ppid)
        .ok()
}

fn get_macos_process_cpu_usage(
    pids: &[Pid],
) -> std::io::Result<std::collections::HashMap<i32, f64>> {
    use itertools::Itertools;
    let output = std::process::Command::new("ps")
        .args(["-o", "pid=,pcpu=", "-p"])
        .arg(
            // Has to look like this since otherwise, it you hit a `unstable_name_collisions` warning.
            Itertools::intersperse(pids.iter().map(i32::to_string), ",".to_string())
                .collect::<String>(),
        )
        .output()?;
    let mut result = std::collections::HashMap::new();
    String::from_utf8_lossy(&output.stdout)
        .split_whitespace()
        .chunks(2)
        .into_iter()
        .for_each(|chunk| {
            let chunk: Vec<&str> = chunk.collect();
            if chunk.len() != 2 {
                panic!("Unexpected `ps` output");
            }
            let pid = chunk[0].parse();
            let usage = chunk[1].parse();
            if let (Ok(pid), Ok(usage)) = (pid, usage) {
                result.insert(pid, usage);
            }
        });
    Ok(result)
}
