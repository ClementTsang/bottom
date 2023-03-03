//! Data collection for temperature metrics.
//!
//! For Linux and macOS, this is handled by Heim.
//! For Windows, this is handled by sysinfo.

cfg_if::cfg_if! {
    if #[cfg(target_os = "linux")] {
        pub mod linux;
        pub use self::linux::*;
    } else if #[cfg(any(target_os = "freebsd", target_os = "macos", target_os = "windows"))] {
        pub mod sysinfo;
        pub use self::sysinfo::*;
    }
}

#[cfg(feature = "nvidia")]
pub mod nvidia;

use crate::app::Filter;

#[derive(Default, Debug, Clone)]
pub struct TempHarvest {
    pub name: String,
    pub temperature: f32,
}

#[derive(Clone, Debug, Copy, PartialEq, Eq)]
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

fn convert_celsius_to_kelvin(celsius: f32) -> f32 {
    celsius + 273.15
}

fn convert_celsius_to_fahrenheit(celsius: f32) -> f32 {
    (celsius * (9.0 / 5.0)) + 32.0
}

fn is_temp_filtered(filter: &Option<Filter>, text: &str) -> bool {
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
