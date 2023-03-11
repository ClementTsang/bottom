//! Disk stats via sysinfo.

use sysinfo::System;

use crate::{app::filter::Filter, utils::error};

use super::DiskHarvest;

pub(crate) fn get_disk_usage(
    sys: &System, disk_filter: &Option<Filter>, mount_filter: &Option<Filter>,
) -> error::Result<Vec<DiskHarvest>> {
    Ok(vec![])
}
