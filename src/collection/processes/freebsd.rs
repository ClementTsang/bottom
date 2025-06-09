//! Process data collection for FreeBSD.  Uses sysinfo.

use std::{io, process::Command};

use hashbrown::HashMap;
use serde::{Deserialize, Deserializer};

use crate::collection::{Pid, deserialize_xo, processes::UnixProcessExt};

#[derive(Deserialize, Debug, Default)]
#[serde(rename_all = "kebab-case")]
struct ProcessInformation {
    process: Vec<ProcessRow>,
}

#[derive(Deserialize, Debug)]
#[serde(rename_all = "kebab-case")]
struct ProcessRow {
    #[serde(deserialize_with = "pid")]
    pid: i32,
    #[serde(deserialize_with = "percent_cpu")]
    percent_cpu: f32,
}

pub(crate) struct FreeBSDProcessExt;

impl UnixProcessExt for FreeBSDProcessExt {
    #[inline]
    fn has_backup_proc_cpu_fn() -> bool {
        true
    }

    fn backup_proc_cpu(pids: &[Pid]) -> io::Result<HashMap<Pid, f32>> {
        if pids.is_empty() {
            return Ok(HashMap::new());
        }

        let output = Command::new("ps")
            .args(["--libxo", "json", "-o", "pid,pcpu", "-p"])
            .args(pids.iter().map(i32::to_string))
            .output()?;

        deserialize_xo("process-information", &output.stdout).map(
            |process_info: ProcessInformation| {
                process_info
                    .process
                    .into_iter()
                    .map(|row| (row.pid, row.percent_cpu))
                    .collect()
            },
        )
    }
}

fn pid<'de, D>(deserializer: D) -> Result<i32, D::Error>
where
    D: Deserializer<'de>,
{
    let s = String::deserialize(deserializer)?;
    s.parse().map_err(serde::de::Error::custom)
}

fn percent_cpu<'de, D>(deserializer: D) -> Result<f32, D::Error>
where
    D: Deserializer<'de>,
{
    let s = String::deserialize(deserializer)?;
    s.parse().map_err(serde::de::Error::custom)
}
