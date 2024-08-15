//! The data collector for FreeBSD.

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

/// The [`DataCollector`] for FreeBSD.
pub struct FreeBsdDataCollector {
    temp_type: TemperatureType,
    temp_filters: Option<Filter>,
}

impl DataCollector for FreeBsdDataCollector {
    fn refresh_data(&mut self) -> CollectionResult<()> {
        Ok(())
    }

    fn get_temperature_data(&self) -> CollectionResult<Option<Vec<TemperatureData>>> {
        let mut results = get_temperature_data(&self.temp_type, &self.temp_filters);

        for entry in sysctl_temp_iter(&self.temp_type, &self.temp_filters) {
            results.push(entry);
        }

        Ok(Some(results))
    }
}
