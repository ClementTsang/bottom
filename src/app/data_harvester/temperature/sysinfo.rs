//! Gets temperature data via sysinfo.

use anyhow::Result;

use super::{
    convert_celsius_to_fahrenheit, convert_celsius_to_kelvin, is_temp_filtered, TempHarvest,
    TemperatureType,
};
use crate::app::Filter;

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

    // For RockPro64 boards on FreeBSD, they apparently use "hw.temperature" for sensors.
    #[cfg(target_os = "freebsd")]
    {
        use sysctl::Sysctl;

        const KEY: &str = "hw.temperature";
        if let Ok(root) = sysctl::Ctl::new(KEY) {
            for ctl in sysctl::CtlIter::below(root).flatten() {
                if let (Ok(name), Ok(temp)) = (ctl.name(), ctl.value()) {
                    if let Some(temp) = temp.as_temperature() {
                        temperature_vec.push(TempHarvest {
                            name,
                            temperature: match temp_type {
                                TemperatureType::Celsius => temp.celsius(),
                                TemperatureType::Kelvin => temp.kelvin(),
                                TemperatureType::Fahrenheit => temp.fahrenheit(),
                            },
                        });
                    }
                }
            }
        }
    }

    // TODO: Should we instead use a hashmap -> vec to skip dupes?
    Ok(Some(temperature_vec))
}
