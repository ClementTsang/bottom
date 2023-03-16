#![allow(unused_imports)] // FIXME: Remove this

use crate::{
    app::{
        data_harvester::disks::{DiskHarvest, IoHarvest},
        filter::Filter,
    },
    utils::error,
};

cfg_if::cfg_if! {
    if #[cfg(target_os = "linux")] {
        mod linux;
        use linux::*;
    } else if #[cfg(not(target_os = "linux"))] {
        mod other;
        use other::*;
    }
}

pub fn get_io_usage() -> error::Result<IoHarvest> {
    Ok(IoHarvest::default())
}

#[allow(dead_code)]
#[allow(unused_variables)]
pub fn get_disk_usage(
    disk_filter: &Option<Filter>, mount_filter: &Option<Filter>,
) -> error::Result<Vec<DiskHarvest>> {
    Ok(vec![])
}
