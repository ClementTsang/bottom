//! Gets temperature data via sysinfo.

use crate::{
    app::filter::Filter,
    new_data_collection::sources::common::temperature::{TemperatureData, TemperatureType},
};

pub fn get_temperature_data(
    components: &sysinfo::Components, temp_type: &TemperatureType, filter: &Option<Filter>,
) -> Vec<TemperatureData> {
    let mut temperature_vec: Vec<TemperatureData> = Vec::new();

    for component in components {
        let name = component.label().to_string();

        if Filter::optional_should_keep(filter, &name) {
            temperature_vec.push(TemperatureData {
                name,
                temperature: Some(temp_type.convert_temp_unit(component.temperature())),
            });
        }
    }

    // TODO: Should we instead use a hashmap -> vec to skip dupes?
    temperature_vec
}
