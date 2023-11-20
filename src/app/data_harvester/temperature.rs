//! Data collection for temperature metrics.
//!
//! For Linux and macOS, this is handled by Heim.
//! For Windows, this is handled by sysinfo.

cfg_if::cfg_if! {
    if #[cfg(target_os = "linux")] {
        pub mod linux;
        pub use self::linux::*;
    } else if #[cfg(any(target_os = "freebsd", target_os = "macos", target_os = "windows", target_os = "android", target_os = "ios"))] {
        pub mod sysinfo;
        pub use self::sysinfo::*;
    }
}

use crate::app::Filter;

#[derive(Default, Debug, Clone)]
pub struct TempHarvest {
    pub name: String,
    pub temperature: f32,
}

#[derive(Clone, Debug, Copy, PartialEq, Eq, Default)]
pub enum TemperatureType {
    #[default]
    Celsius,
    Kelvin,
    Fahrenheit,
}

impl TemperatureType {
    /// Given a temperature in Celsius, covert it if necessary for a different unit.
    pub fn convert_temp_unit(&self, temp_celsius: f32) -> f32 {
        fn convert_celsius_to_kelvin(celsius: f32) -> f32 {
            celsius + 273.15
        }

        fn convert_celsius_to_fahrenheit(celsius: f32) -> f32 {
            (celsius * (9.0 / 5.0)) + 32.0
        }

        match self {
            TemperatureType::Celsius => temp_celsius,
            TemperatureType::Kelvin => convert_celsius_to_kelvin(temp_celsius),
            TemperatureType::Fahrenheit => convert_celsius_to_fahrenheit(temp_celsius),
        }
    }
}

pub fn is_temp_filtered(filter: &Option<Filter>, text: &str) -> bool {
    if let Some(filter) = filter {
        let mut ret = filter.is_list_ignored;
        for r in &filter.list {
            if r.is_match(text) {
                ret = !filter.is_list_ignored;
                break;
            }
        }
        ret
    } else {
        true
    }
}

#[cfg(test)]
mod test {
    use crate::app::data_harvester::temperature::TemperatureType;

    #[test]
    fn temp_conversions() {
        const TEMP: f32 = 100.0;

        assert_eq!(
            TemperatureType::Celsius.convert_temp_unit(TEMP),
            TEMP,
            "celsius to celsius is the same"
        );

        assert_eq!(TemperatureType::Kelvin.convert_temp_unit(TEMP), 373.15);

        assert_eq!(TemperatureType::Fahrenheit.convert_temp_unit(TEMP), 212.0);
    }
}
