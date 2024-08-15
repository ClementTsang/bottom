//! FreeBSD-specific temperature extraction code.

// For RockPro64 boards on FreeBSD, they apparently use "hw.temperature" for
// sensors.
use sysctl::Sysctl;

/// Return an iterator of temperature data pulled from sysctl.
pub(crate) fn sysctl_temp_iter(
    temp_type: &TemperatureType, filter: &Option<Filter>,
) -> impl Iterator<Item = TemperatureData> {
    const KEY: &str = "hw.temperature";

    if let Ok(root) = sysctl::Ctl::new(KEY) {
        sysctl::CtlIter::below(root).flatten().filter_map(|ctl| {
            if let (Ok(name), Ok(temp)) = (ctl.name(), ctl.value()) {
                if let Some(temp) = temp.as_temperature() {
                    if Filter::optional_should_keep(filter, &name) {
                        return Some(TemperatureData {
                            name,
                            temperature: Some(match temp_type {
                                TemperatureType::Celsius => temp.celsius(),
                                TemperatureType::Kelvin => temp.kelvin(),
                                TemperatureType::Fahrenheit => temp.fahrenheit(),
                            }),
                        });
                    }
                }
            }

            None
        })
    } else {
        std::iter::empty()
    }
}
