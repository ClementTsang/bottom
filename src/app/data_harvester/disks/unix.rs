//! Implementation based on [heim's](https://github.com/heim-rs/heim)
//! Unix disk usage.

use std::path::Path;

use crate::app::{
    data_harvester::disks::{DiskHarvest, IoHarvest},
    filter::Filter,
};

mod file_systems;
pub(crate) use file_systems::FileSystem;

cfg_if::cfg_if! {
    if #[cfg(target_os = "linux")] {
        use super::linux::partitions;
    } else {
        mod partition;
        mod mounts;

        use partition::*;
    }
}

pub(crate) struct Usage {}

fn disk_usage(path: &Path) -> anyhow::Result<Usage> {
    todo!()
}

pub fn get_disk_usage(
    disk_filter: &Option<Filter>, mount_filter: &Option<Filter>,
) -> anyhow::Result<Vec<DiskHarvest>> {
    let partitions = partitions()?;

    Ok(vec![])
}

pub fn get_io_usage() -> anyhow::Result<IoHarvest> {
    Ok(IoHarvest::default())
}
