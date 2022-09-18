//! Gets temperature data via sysinfo.

use super::{
    convert_celsius_to_fahrenheit, convert_celsius_to_kelvin, is_temp_filtered, temp_vec_sort,
    TempHarvest, TemperatureType,
};
use crate::app::Filter;
use anyhow::Result;

pub fn get_temperature_data(
    sys: &sysinfo::System, temp_type: &TemperatureType, filter: &Option<Filter>,
) -> Result<Option<Vec<TempHarvest>>> {
    use sysinfo::{ComponentExt, SystemExt};

    let mut temperature_vec: Vec<TempHarvest> = Vec::new();

    let sensor_data = sys.components();
    for component in sensor_data {
        let name = component.label().to_string();

        if is_temp_filtered(filter, &name) {
            temperature_vec.push(TempHarvest {
                name,
                temperature: match temp_type {
                    TemperatureType::Celsius => component.temperature(),
                    TemperatureType::Kelvin => convert_celsius_to_kelvin(component.temperature()),
                    TemperatureType::Fahrenheit => {
                        convert_celsius_to_fahrenheit(component.temperature())
                    }
                },
            });
        }
    }

    #[cfg(feature = "nvidia")]
    {
        super::nvidia::add_nvidia_data(&mut temperature_vec, temp_type, filter)?;
    }

    temp_vec_sort(&mut temperature_vec);
    Ok(Some(temperature_vec))
}
