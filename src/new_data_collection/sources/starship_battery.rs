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
    Battery, Manager, State,
};

#[derive(Debug, Clone)]
pub struct BatteryHarvest {
    pub charge_percent: f64,
    pub secs_until_full: Option<i64>,
    pub secs_until_empty: Option<i64>,
    pub power_consumption_rate_watts: f64,
    pub health_percent: f64,
    pub state: State,
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
                    state: battery.state(),
                })
            } else {
                None
            }
        })
        .collect::<Vec<_>>()
}
