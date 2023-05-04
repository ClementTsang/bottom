//! Process data collection for FreeBSD.  Uses sysinfo.

use std::io;

use hashbrown::HashMap;
use serde::{Deserialize, Deserializer};
use sysinfo::System;

use super::ProcessHarvest;
use crate::data_harvester::deserialize_xo;
use crate::data_harvester::processes::UserTable;

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
    percent_cpu: f64,
}

pub fn get_process_data(
    sys: &System, use_current_cpu_total: bool, unnormalized_cpu: bool, total_memory: u64,
    user_table: &mut UserTable,
) -> crate::utils::error::Result<Vec<ProcessHarvest>> {
    super::unix::process_data_with_backup(
        sys,
        use_current_cpu_total,
        unnormalized_cpu,
        total_memory,
        user_table,
        get_freebsd_process_cpu_usage,
    )
}

fn get_freebsd_process_cpu_usage(pids: &[i32]) -> io::Result<HashMap<i32, f64>> {
    if pids.is_empty() {
        return Ok(HashMap::new());
    }

    let output = std::process::Command::new("ps")
        .args(["--libxo", "json", "-o", "pid,pcpu", "-p"])
        .args(pids.iter().map(i32::to_string))
        .output()?;
    deserialize_xo("process-information", &output.stdout).map(|process_info: ProcessInformation| {
        process_info
            .process
            .into_iter()
            .map(|row| (row.pid, row.percent_cpu))
            .collect()
    })
}

fn pid<'de, D>(deserializer: D) -> Result<i32, D::Error>
where
    D: Deserializer<'de>,
{
    let s = String::deserialize(deserializer)?;
    s.parse().map_err(serde::de::Error::custom)
}

fn percent_cpu<'de, D>(deserializer: D) -> Result<f64, D::Error>
where
    D: Deserializer<'de>,
{
    let s = String::deserialize(deserializer)?;
    s.parse().map_err(serde::de::Error::custom)
}
