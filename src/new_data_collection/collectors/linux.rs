//! The data collector for Linux.

use std::time::Instant;

use crate::{
    app::filter::Filter,
    data_collection::Data,
    new_data_collection::{
        error::CollectionResult,
        sources::{
            cpu::CpuHarvest,
            disk::DiskHarvest,
            linux::{get_temperature_data, linux_process_data, ProcessCollector},
            memory::MemHarvest,
            processes::ProcessHarvest,
            sysinfo::{
                cpu::{get_cpu_data_list, get_load_avg},
                memory::{get_cache_usage, get_ram_usage, get_swap_usage},
            },
            temperature::{TemperatureData, TemperatureType},
        },
    },
};

use super::common::DataCollector;

cfg_if::cfg_if! {
    if #[cfg(feature = "battery")] {
        use starship_battery::{Battery, Manager};
        use crate::new_data_collection::sources::battery::BatteryHarvest;
    }

}

/// The [`DataCollector`] for Linux.
pub struct LinuxDataCollector {
    current_collection_time: Instant,
    last_collection_time: Instant,

    temp_type: TemperatureType,
    temp_filters: Option<Filter>,

    proc_collector: ProcessCollector,

    system: sysinfo::System,
    network: sysinfo::Networks,

    show_average_cpu: bool,

    #[cfg(feature = "battery")]
    batteries: Option<(Manager, Vec<Battery>)>,

    #[cfg(feature = "gpu")]
    nvml: nvml_wrapper::Nvml,

    #[cfg(feature = "gpu")]
    gpus_total_mem: Option<u64>,
}

impl LinuxDataCollector {
    fn refresh_data(&mut self) -> CollectionResult<()> {
        Ok(())
    }
}

impl DataCollector for LinuxDataCollector {
    fn get_data(&mut self) -> Data {
        let collection_time = Instant::now();

        todo!()
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

    fn get_disk_data(&mut self) -> CollectionResult<DiskHarvest> {
        todo!()
    }

    fn get_cpu_data(&mut self) -> CollectionResult<CpuHarvest> {
        let usages = get_cpu_data_list(&self.system, self.show_average_cpu);
        let load_average = get_load_avg();

        CollectionResult::Ok(CpuHarvest {
            usages,
            load_average,
        })
    }

    fn get_memory_data(&mut self) -> CollectionResult<MemHarvest> {
        let memory = get_ram_usage(&self.system);
        let swap = get_swap_usage(&self.system);
        let cache = get_cache_usage(&self.system);

        CollectionResult::Ok(MemHarvest {
            memory,
            swap,
            cache,
            #[cfg(feature = "zfs")]
            arc: crate::new_data_collection::sources::linux::get_arc_usage(),
            #[cfg(feature = "gpu")]
            gpu: crate::new_data_collection::sources::nvidia::get_gpu_memory_usage(&self.nvml),
        })
    }

    #[cfg(feature = "battery")]
    fn get_battery_data(&mut self) -> CollectionResult<Vec<BatteryHarvest>> {
        use crate::new_data_collection::{
            error::CollectionError, sources::starship::refresh_batteries,
        };

        match &mut self.batteries {
            Some((battery_manager, battery_list)) => {
                CollectionResult::Ok(refresh_batteries(battery_manager, battery_list))
            }
            None => CollectionResult::Err(CollectionError::NoData),
        }
    }
}
