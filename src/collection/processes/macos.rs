//! Process data collection for macOS.  Uses sysinfo and custom bindings.

mod sysctl_bindings;

use std::{io, process::Command};

use hashbrown::HashMap;
use itertools::Itertools;

use super::UnixProcessExt;
use crate::collection::Pid;

pub(crate) struct MacOSProcessExt;

impl UnixProcessExt for MacOSProcessExt {
    #[inline]
    fn has_backup_proc_cpu_fn() -> bool {
        true
    }

    fn backup_proc_cpu(pids: &[Pid]) -> io::Result<HashMap<Pid, f32>> {
        let output = Command::new("ps")
            .args(["-o", "pid=,pcpu=", "-p"])
            .arg(
                // Has to look like this since otherwise, it you hit a `unstable_name_collisions`
                // warning.
                Itertools::intersperse(pids.iter().map(i32::to_string), ",".to_string())
                    .collect::<String>(),
            )
            .output()?;
        let mut result = HashMap::new();
        String::from_utf8_lossy(&output.stdout)
            .split_whitespace()
            .chunks(2)
            .into_iter()
            .for_each(|chunk| {
                let chunk: Vec<&str> = chunk.collect();
                if chunk.len() != 2 {
                    panic!("Unexpected 'ps' output");
                }
                let pid = chunk[0].parse();
                let usage = chunk[1].parse();
                if let (Ok(pid), Ok(usage)) = (pid, usage) {
                    result.insert(pid, usage);
                }
            });
        Ok(result)
    }

    fn parent_pid(process_val: &sysinfo::Process) -> Option<Pid> {
        process_val
            .parent()
            .map(|p| p.as_u32() as _)
            .or_else(|| fallback_macos_ppid(process_val.pid().as_u32() as _))
    }
}

fn fallback_macos_ppid(pid: Pid) -> Option<Pid> {
    sysctl_bindings::kinfo_process(pid)
        .map(|kinfo| kinfo.kp_eproc.e_ppid)
        .ok()
}
