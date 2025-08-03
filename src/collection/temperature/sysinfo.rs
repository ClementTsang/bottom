//! Gets temperature data via sysinfo.

use anyhow::Result;

use super::TempSensorData;
use crate::app::filter::Filter;

pub fn get_temperature_data(
    components: &sysinfo::Components, filter: &Option<Filter>,
) -> Result<Option<Vec<TempSensorData>>> {
    let mut temperatures: Vec<TempSensorData> = Vec::new();

    for component in components {
        let name = component.label().to_string();

        if Filter::optional_should_keep(filter, &name) {
            temperatures.push(TempSensorData {
                name,
                temperature: component.temperature(),
            });
        }
    }

    // For RockPro64 boards on FreeBSD, they apparently use "hw.temperature" for
    // sensors.
    #[cfg(target_os = "freebsd")]
    {
        use sysctl::Sysctl;

        const KEY: &str = "hw.temperature";
        if let Ok(root) = sysctl::Ctl::new(KEY) {
            for ctl in sysctl::CtlIter::below(root).flatten() {
                if let (Ok(name), Ok(temp)) = (ctl.name(), ctl.value()) {
                    if let Some(temp) = temp.as_temperature() {
                        temperatures.push(TempSensorData {
                            name,
                            temperature: Some(temp.celsius()),
                        });
                    }
                }
            }
        }
    }

    // TODO: Should we instead use a hashmap -> vec to skip dupes?
    Ok(Some(temperatures))
}
