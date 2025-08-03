//! Code around temperature data.

use std::{fmt::Display, str::FromStr};

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
    pub fn convert_temp_unit(&self, celsius: f32) -> TypedTemperature {
        match self {
            TemperatureType::Celsius => TypedTemperature::Celsius(celsius.ceil() as u32),
            TemperatureType::Kelvin => TypedTemperature::Kelvin((celsius + 273.15).ceil() as u32),
            TemperatureType::Fahrenheit => {
                TypedTemperature::Fahrenheit(((celsius * (9.0 / 5.0)) + 32.0).ceil() as u32)
            }
        }
    }
}

/// A temperature and its type.
#[derive(Debug, PartialEq, Clone, Eq, PartialOrd, Ord)]
pub enum TypedTemperature {
    Celsius(u32),
    Kelvin(u32),
    Fahrenheit(u32),
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
    use super::*;

    #[test]
    fn temp_conversions() {
        const TEMP: f32 = 100.0;

        assert_eq!(
            TemperatureType::Celsius.convert_temp_unit(TEMP),
            TypedTemperature::Celsius(TEMP as u32),
        );

        assert_eq!(
            TemperatureType::Kelvin.convert_temp_unit(TEMP),
            TypedTemperature::Kelvin(373.15_f32.ceil() as u32)
        );

        assert_eq!(
            TemperatureType::Fahrenheit.convert_temp_unit(TEMP),
            TypedTemperature::Fahrenheit(212)
        );
    }
}
