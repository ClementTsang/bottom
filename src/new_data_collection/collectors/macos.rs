//! The data collector for macOS.

use crate::{
    app::filter::Filter,
    new_data_collection::{
        error::CollectionResult,
        sources::{
            common::temperature::{TemperatureData, TemperatureType},
            sysinfo::temperature::get_temperature_data,
        },
    },
};

use super::common::DataCollector;

/// The [`DataCollector`] for macOS.
pub struct MacOsDataCollector {
    temp_type: TemperatureType,
    temp_filters: Option<Filter>,
}

impl DataCollector for MacOsDataCollector {
    fn refresh_data(&mut self) -> CollectionResult<()> {
        Ok(())
    }

    fn get_temperature_data(&self) -> CollectionResult<Option<Vec<TemperatureData>>> {
        Ok(Some(get_temperature_data(
            &self.temp_type,
            &self.temp_filters,
        )))
    }
}
