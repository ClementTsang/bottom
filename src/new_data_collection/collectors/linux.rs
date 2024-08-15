//! The data collector for Linux.

use std::time::Instant;

use starship_battery::{Battery, Manager};

use crate::{
    app::filter::Filter,
    new_data_collection::{
        error::CollectionResult,
        sources::{
            common::{
                processes::ProcessHarvest,
                temperature::{TemperatureData, TemperatureType},
            },
            linux::{
                processes::{linux_process_data, ProcessCollector},
                temperature::get_temperature_data,
            },
        },
    },
};

use super::common::DataCollector;

/// The [`DataCollector`] for Linux.
pub struct LinuxDataCollector {
    current_collection_time: Instant,
    last_collection_time: Instant,

    temp_type: TemperatureType,
    temp_filters: Option<Filter>,

    proc_collector: ProcessCollector,

    system: sysinfo::System,
    network: sysinfo::Networks,

    #[cfg(feature = "battery")]
    battery_manager: Option<Manager>,
    #[cfg(feature = "battery")]
    battery_list: Option<Vec<Battery>>,

    #[cfg(feature = "gpu")]
    gpus_total_mem: Option<u64>,
}

impl DataCollector for LinuxDataCollector {
    fn refresh_data(&mut self) -> CollectionResult<()> {
        Ok(())
    }

    fn get_temperature_data(&mut self) -> CollectionResult<Vec<TemperatureData>> {
        Ok(get_temperature_data(&self.temp_type, &self.temp_filters))
    }

    fn get_process_data(&mut self) -> CollectionResult<Vec<ProcessHarvest>> {
        let time_diff = self
            .current_collection_time
            .duration_since(self.last_collection_time)
            .as_secs();

        linux_process_data(
            &self.system,
            time_diff,
            &mut self.proc_collector,
            #[cfg(feature = "gpu")]
            self.gpus_total_mem,
        )
    }
}
