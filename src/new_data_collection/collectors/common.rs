//! Common code amongst all data collectors.

use crate::{
    data_collection::Data,
    new_data_collection::{
        error::CollectionResult,
        sources::{
            cpu::CpuHarvest, disk::DiskHarvest, memory::MemHarvest, processes::ProcessHarvest,
            temperature::TemperatureData,
        },
    },
};

#[cfg(feature = "battery")]
use crate::new_data_collection::sources::battery::BatteryHarvest;

// /// Represents data collected at an instance.
// #[derive(Debug)]
// pub struct Data {
//     pub collection_time: Instant,
//     pub temperature_data: Option<Vec<TemperatureData>>,
//     pub process_data: Option<Vec<ProcessHarvest>>,
//     pub disk_data: Option<DiskHarvest>,
// }

/// The trait representing what a per-platform data collector should implement.
pub trait DataCollector {
    /// Return data.
    ///
    /// For now, this returns the old data type for cross-compatibility as we migrate.
    fn get_data(&mut self) -> Data;

    /// Return temperature data.
    fn get_temperature_data(&mut self) -> CollectionResult<Vec<TemperatureData>>;

    /// Return process data.
    fn get_process_data(&mut self) -> CollectionResult<Vec<ProcessHarvest>>;

    /// Return disk data.
    fn get_disk_data(&mut self) -> CollectionResult<DiskHarvest>;

    /// Return CPU data.
    fn get_cpu_data(&mut self) -> CollectionResult<CpuHarvest>;

    /// Return memory data.
    fn get_memory_data(&mut self) -> CollectionResult<MemHarvest>;

    #[cfg(feature = "battery")]
    /// Return battery data.
    fn get_battery_data(&mut self) -> CollectionResult<Vec<BatteryHarvest>>;
}
