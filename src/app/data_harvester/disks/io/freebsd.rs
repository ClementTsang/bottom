use crate::{app::data_harvester::disks::IoHarvest, utils::error};

pub fn get_io_usage() -> error::Result<IoHarvest> {
    let io_harvest = get_disk_info().map(|storage_system_information| {
        storage_system_information
            .filesystem
            .into_iter()
            .map(|disk| (disk.name, None))
            .collect()
    })?;

    Ok(io_harvest)
}
