//! Gets temperature data via heim.

use super::{is_temp_filtered, temp_vec_sort, TempHarvest, TemperatureType};
use crate::app::Filter;

pub async fn get_temperature_data(
    temp_type: &TemperatureType, actually_get: bool, filter: &Option<Filter>,
) -> crate::utils::error::Result<Option<Vec<TempHarvest>>> {
    use futures::StreamExt;
    use heim::units::thermodynamic_temperature;

    if !actually_get {
        return Ok(None);
    }

    let mut temperature_vec: Vec<TempHarvest> = Vec::new();

    let mut sensor_data = heim::sensors::temperatures().boxed_local();
    while let Some(sensor) = sensor_data.next().await {
        if let Ok(sensor) = sensor {
            let component_name = Some(sensor.unit().to_string());
            let component_label = sensor.label().map(|label| label.to_string());

            let name = match (component_name, component_label) {
                (Some(name), Some(label)) => format!("{}: {}", name, label),
                (None, Some(label)) => label.to_string(),
                (Some(name), None) => name.to_string(),
                (None, None) => String::default(),
            };

            if is_temp_filtered(filter, &name) {
                temperature_vec.push(TempHarvest {
                    name,
                    temperature: match temp_type {
                        TemperatureType::Celsius => sensor
                            .current()
                            .get::<thermodynamic_temperature::degree_celsius>(
                        ),
                        TemperatureType::Kelvin => {
                            sensor.current().get::<thermodynamic_temperature::kelvin>()
                        }
                        TemperatureType::Fahrenheit => sensor
                            .current()
                            .get::<thermodynamic_temperature::degree_fahrenheit>(
                        ),
                    },
                });
            }
        }
    }

    temp_vec_sort(&mut temperature_vec);
    Ok(Some(temperature_vec))
}
