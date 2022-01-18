//! Gets temperature data via sysinfo.

use super::{is_temp_filtered, temp_vec_sort, TempHarvest, TemperatureType};
use crate::app::Filter;

pub async fn get_temperature_data(
    sys: &sysinfo::System, temp_type: &TemperatureType, actually_get: bool, filter: &Option<Filter>,
) -> crate::utils::error::Result<Option<Vec<TempHarvest>>> {
    use sysinfo::{ComponentExt, SystemExt};

    if !actually_get {
        return Ok(None);
    }

    fn convert_celsius_to_kelvin(celsius: f32) -> f32 {
        celsius + 273.15
    }

    fn convert_celsius_to_fahrenheit(celsius: f32) -> f32 {
        (celsius * (9.0 / 5.0)) + 32.0
    }

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

    temp_vec_sort(&mut temperature_vec);
    Ok(Some(temperature_vec))
}
