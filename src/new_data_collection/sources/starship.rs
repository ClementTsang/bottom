//! Covers battery usage for:
//! - Linux 2.6.39+
//! - MacOS 10.10+
//! - iOS
//! - Windows 7+
//! - FreeBSD
//! - DragonFlyBSD
//!
//! For more information, refer to the [starship_battery](https://github.com/starship/rust-battery) repo/docs.

use starship_battery::{
    units::{power::watt, ratio::percent, time::second},
    Battery, Manager,
};

use super::battery::{BatteryHarvest, State};

impl From<starship_battery::State> for State {
    fn from(value: starship_battery::State) -> Self {
        match value {
            starship_battery::State::Unknown => State::Unknown,
            starship_battery::State::Charging => State::Charging,
            starship_battery::State::Discharging => State::Discharging,
            starship_battery::State::Empty => State::Empty,
            starship_battery::State::Full => State::Full,
        }
    }
}

pub fn refresh_batteries(manager: &Manager, batteries: &mut [Battery]) -> Vec<BatteryHarvest> {
    batteries
        .iter_mut()
        .filter_map(|battery| {
            if manager.refresh(battery).is_ok() {
                Some(BatteryHarvest {
                    secs_until_full: {
                        let optional_time = battery.time_to_full();
                        optional_time.map(|time| f64::from(time.get::<second>()) as i64)
                    },
                    secs_until_empty: {
                        let optional_time = battery.time_to_empty();
                        optional_time.map(|time| f64::from(time.get::<second>()) as i64)
                    },
                    charge_percent: f64::from(battery.state_of_charge().get::<percent>()),
                    power_consumption_rate_watts: f64::from(battery.energy_rate().get::<watt>()),
                    health_percent: f64::from(battery.state_of_health().get::<percent>()),
                    state: battery.state().into(),
                })
            } else {
                None
            }
        })
        .collect::<Vec<_>>()
}
