use std::cmp::Ordering;

use futures::StreamExt;
use heim::units::thermodynamic_temperature;
use sysinfo::{ComponentExt, System, SystemExt};

#[derive(Default, Debug, Clone)]
pub struct TempHarvest {
    pub component_name: String,
    pub temperature: f32,
}

#[derive(Clone, Debug)]
pub enum TemperatureType {
    Celsius,
    Kelvin,
    Fahrenheit,
}

impl Default for TemperatureType {
    fn default() -> Self {
        TemperatureType::Celsius
    }
}

pub async fn get_temperature_data(
    sys: &System, temp_type: &TemperatureType, actually_get: bool,
) -> crate::utils::error::Result<Option<Vec<TempHarvest>>> {
    if !actually_get {
        return Ok(None);
    }

    let mut temperature_vec: Vec<TempHarvest> = Vec::new();

    if cfg!(target_os = "linux") {
        let mut sensor_data = heim::sensors::temperatures();
        while let Some(sensor) = sensor_data.next().await {
            if let Ok(sensor) = sensor {
                temperature_vec.push(TempHarvest {
                    component_name: sensor.unit().to_string(),
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
    } else {
        let sensor_data = sys.get_components();
        for component in sensor_data {
            temperature_vec.push(TempHarvest {
                component_name: component.get_label().to_string(),
                temperature: match temp_type {
                    TemperatureType::Celsius => component.get_temperature(),
                    TemperatureType::Kelvin => {
                        convert_celsius_to_kelvin(component.get_temperature())
                    }
                    TemperatureType::Fahrenheit => {
                        convert_celsius_to_fahrenheit(component.get_temperature())
                    }
                },
            });
        }
    }

    // By default, sort temperature, then by alphabetically!

    // Note we sort in reverse here; we want greater temps to be higher priority.
    temperature_vec.sort_by(|a, b| match a.temperature.partial_cmp(&b.temperature) {
        Some(x) => match x {
            Ordering::Less => Ordering::Greater,
            Ordering::Greater => Ordering::Less,
            Ordering::Equal => Ordering::Equal,
        },
        None => Ordering::Equal,
    });

    temperature_vec.sort_by(|a, b| {
        a.component_name
            .partial_cmp(&b.component_name)
            .unwrap_or(Ordering::Equal)
    });

    Ok(Some(temperature_vec))
}

fn convert_celsius_to_kelvin(celsius: f32) -> f32 {
    celsius + 273.15
}

fn convert_celsius_to_fahrenheit(celsius: f32) -> f32 {
    (celsius * (9.0 / 5.0)) + 32.0
}
