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

use std::{fmt::Display, str::FromStr};

#[derive(Default, Debug, Clone)]
pub struct TempHarvest {
    pub name: String,
    pub temperature: Option<TypedTemperature>,
}

#[derive(Clone, Debug, Copy, PartialEq, Eq, Default)]
pub enum TemperatureType {
    #[default]
    Celsius,
    Kelvin,
    Fahrenheit,
}

impl FromStr for TemperatureType {
    type Err = String;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s {
            "fahrenheit" | "f" => Ok(TemperatureType::Fahrenheit),
            "kelvin" | "k" => Ok(TemperatureType::Kelvin),
            "celsius" | "c" => Ok(TemperatureType::Celsius),
            _ => Err(format!(
                "'{s}' is an invalid temperature type, use one of: [kelvin, k, celsius, c, fahrenheit, f]."
            )),
        }
    }
}

impl TemperatureType {
    /// Given a temperature in Celsius, covert it if necessary for a different
    /// unit.
    pub fn convert_temp_unit(&self, temp_celsius: f32) -> TypedTemperature {
        fn celsius_to_kelvin(celsius: f32) -> TypedTemperature {
            TypedTemperature::Kelvin(celsius + 273.15)
        }

        fn celsius_to_fahrenheit(celsius: f32) -> TypedTemperature {
            TypedTemperature::Fahrenheit((celsius * (9.0 / 5.0)) + 32.0)
        }

        match self {
            TemperatureType::Celsius => TypedTemperature::Celsius(temp_celsius),
            TemperatureType::Kelvin => celsius_to_kelvin(temp_celsius),
            TemperatureType::Fahrenheit => celsius_to_fahrenheit(temp_celsius),
        }
    }
}

/// A temperature and its type.
#[derive(Debug, PartialEq, Clone)]
pub enum TypedTemperature {
    Celsius(f32),
    Kelvin(f32),
    Fahrenheit(f32),
}

/// A rounded temperature and its type.
///
/// TODO: (points_rework_v1) this is kinda a hack, but it does work for now...
#[derive(Debug, PartialEq, Eq, Clone, PartialOrd, Ord)]
pub enum RoundedTypedTemperature {
    Celsius(u32),
    Kelvin(u32),
    Fahrenheit(u32),
}

impl From<TypedTemperature> for RoundedTypedTemperature {
    fn from(value: TypedTemperature) -> Self {
        match value {
            TypedTemperature::Celsius(val) => RoundedTypedTemperature::Celsius(val.ceil() as u32),
            TypedTemperature::Kelvin(val) => RoundedTypedTemperature::Kelvin(val.ceil() as u32),
            TypedTemperature::Fahrenheit(val) => {
                RoundedTypedTemperature::Fahrenheit(val.ceil() as u32)
            }
        }
    }
}

impl Display for RoundedTypedTemperature {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            RoundedTypedTemperature::Celsius(val) => write!(f, "{val}°C"),
            RoundedTypedTemperature::Kelvin(val) => write!(f, "{val}K"),
            RoundedTypedTemperature::Fahrenheit(val) => write!(f, "{val}°F"),
        }
    }
}

impl Display for TypedTemperature {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            TypedTemperature::Celsius(val) => write!(f, "{val}°C"),
            TypedTemperature::Kelvin(val) => write!(f, "{val}K"),
            TypedTemperature::Fahrenheit(val) => write!(f, "{val}°F"),
        }
    }
}

#[cfg(test)]
mod test {
    use crate::data_collection::temperature::{TemperatureType, TypedTemperature};

    #[test]
    fn temp_conversions() {
        const TEMP: f32 = 100.0;

        assert_eq!(
            TemperatureType::Celsius.convert_temp_unit(TEMP),
            TypedTemperature::Celsius(TEMP),
        );

        assert_eq!(
            TemperatureType::Kelvin.convert_temp_unit(TEMP),
            TypedTemperature::Kelvin(373.15)
        );

        assert_eq!(
            TemperatureType::Fahrenheit.convert_temp_unit(TEMP),
            TypedTemperature::Fahrenheit(212.0)
        );
    }
}
