//! Data collection for temperature metrics.
//!
//! For Linux and macOS, this is handled by Heim.
//! For Windows, this is handled by sysinfo.

cfg_if::cfg_if! {
    if #[cfg(target_os = "linux")] {
        pub mod heim;
        pub use self::heim::*;
    } else if #[cfg(any(target_os = "macos", target_os = "windows"))] {
        pub mod sysinfo;
        pub use self::sysinfo::*;
    }
}

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
