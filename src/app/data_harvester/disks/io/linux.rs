use crate::{app::data_harvester::disks::IoHarvest, utils::error};

pub fn get_io_usage() -> error::Result<IoHarvest> {
    Ok(IoHarvest::default())
}
