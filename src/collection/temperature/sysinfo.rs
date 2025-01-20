//! Gets temperature data via sysinfo.

use anyhow::Result;

use super::{TempHarvest, TemperatureType};
use crate::app::filter::Filter;

pub fn get_temperature_data(
    components: &sysinfo::Components, temp_type: &TemperatureType, filter: &Option<Filter>,
) -> Result<Option<Vec<TempHarvest>>> {
    let mut temperatures: Vec<TempHarvest> = Vec::new();

    for component in components {
        let name = component.label().to_string();

        if Filter::optional_should_keep(filter, &name) {
            temperatures.push(TempHarvest {
                name,
                temperature: Some(temp_type.convert_temp_unit(component.temperature())),
            });
        }
    }

    // For RockPro64 boards on FreeBSD, they apparently use "hw.temperature" for
    // sensors.
    #[cfg(target_os = "freebsd")]
    {
        use super::TypedTemperature;
        use sysctl::Sysctl;

        const KEY: &str = "hw.temperature";
        if let Ok(root) = sysctl::Ctl::new(KEY) {
            for ctl in sysctl::CtlIter::below(root).flatten() {
                if let (Ok(name), Ok(temp)) = (ctl.name(), ctl.value()) {
                    if let Some(temp) = temp.as_temperature() {
                        temperatures.push(TempHarvest {
                            name,
                            temperature: Some(match temp_type {
                                TemperatureType::Celsius => {
                                    TypedTemperature::Celsius(temp.celsius())
                                }
                                TemperatureType::Kelvin => TypedTemperature::Kelvin(temp.kelvin()),
                                TemperatureType::Fahrenheit => {
                                    TypedTemperature::Fahrenheit(temp.fahrenheit())
                                }
                            }),
                        });
                    }
                }
            }
        }
    }

    // TODO: Should we instead use a hashmap -> vec to skip dupes?
    Ok(Some(temperatures))
}
