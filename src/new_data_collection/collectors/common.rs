//! Common code amongst all data collectors.

use crate::new_data_collection::{
    error::CollectionResult,
    sources::common::{processes::ProcessHarvest, temperature::TemperatureData},
};

/// The trait representing what a per-platform data collector should implement.
pub(crate) trait DataCollector {
    /// Refresh inner data sources to prepare them for gathering data.
    ///
    /// Note that depending on the implementation, this may
    /// not actually need to do anything.
    fn refresh_data(&mut self) -> CollectionResult<()> {
        Ok(())
    }

    /// Return temperature data.
    fn get_temperature_data(&mut self) -> CollectionResult<Vec<TemperatureData>> {
        Ok(vec![])
    }

    /// Return process data.
    fn get_process_data(&mut self) -> CollectionResult<Vec<ProcessHarvest>> {
        Ok(vec![])
    }
}
