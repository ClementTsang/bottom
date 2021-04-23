use std::cmp::Ordering;

use crate::app::Filter;

#[derive(Default, Debug, Clone)]
pub struct TempHarvest {
    pub name: String,
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

fn is_temp_filtered(filter: &Option<Filter>, text: &str) -> bool {
    if let Some(filter) = filter {
        if filter.is_list_ignored {
            let mut ret = true;
            for r in &filter.list {
                if r.is_match(text) {
                    ret = false;
                    break;
                }
            }
            ret
        } else {
            true
        }
    } else {
        true
    }
}

#[cfg(not(target_os = "linux"))]
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

    let sensor_data = sys.get_components();
    for component in sensor_data {
        let name = component.get_label().to_string();

        if is_temp_filtered(filter, &name) {
            temperature_vec.push(TempHarvest {
                name,
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

    temp_vec_sort(&mut temperature_vec);
    Ok(Some(temperature_vec))
}

#[cfg(target_os = "linux")]
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

fn temp_vec_sort(temperature_vec: &mut Vec<TempHarvest>) {
    // By default, sort temperature, then by alphabetically!
    // TODO: [TEMPS] Allow users to control this.

    // Note we sort in reverse here; we want greater temps to be higher priority.
    temperature_vec.sort_by(|a, b| match a.temperature.partial_cmp(&b.temperature) {
        Some(x) => match x {
            Ordering::Less => Ordering::Greater,
            Ordering::Greater => Ordering::Less,
            Ordering::Equal => Ordering::Equal,
        },
        None => Ordering::Equal,
    });

    temperature_vec.sort_by(|a, b| a.name.partial_cmp(&b.name).unwrap_or(Ordering::Equal));
}
