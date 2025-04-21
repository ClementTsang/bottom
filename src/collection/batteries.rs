//! Uses the battery crate.
//!
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
    Battery, Manager, State,
    units::{power::watt, ratio::percent, time::second},
};

/// Battery state.
#[derive(Debug, Clone)]
pub enum BatteryState {
    Charging {
        /// Time to full in seconds.
        time_to_full: Option<u32>,
    },
    Discharging {
        /// Time to empty in seconds.
        time_to_empty: Option<u32>,
    },
    Empty,
    Full,
    Unknown,
}

impl BatteryState {
    /// Return the string representation.
    pub fn as_str(&self) -> &'static str {
        match self {
            BatteryState::Charging { .. } => "Charging",
            BatteryState::Discharging { .. } => "Discharging",
            BatteryState::Empty => "Empty",
            BatteryState::Full => "Full",
            BatteryState::Unknown => "Unknown",
        }
    }
}

#[derive(Debug, Clone)]
pub struct BatteryData {
    /// Current charge percent.
    pub charge_percent: f64,
    /// Power consumption, in watts.
    pub power_consumption: f64,
    /// Reported battery health.
    pub health_percent: f64,
    /// The current battery "state" (e.g. is it full, charging, etc.).
    pub state: BatteryState,
}

impl BatteryData {
    pub fn watt_consumption(&self) -> String {
        format!("{:.2}W", self.power_consumption)
    }

    pub fn health(&self) -> String {
        format!("{:.2}%", self.health_percent)
    }
}

pub fn refresh_batteries(manager: &Manager, batteries: &mut [Battery]) -> Vec<BatteryData> {
    batteries
        .iter_mut()
        .filter_map(|battery| {
            if manager.refresh(battery).is_ok() {
                Some(BatteryData {
                    charge_percent: f64::from(battery.state_of_charge().get::<percent>()),
                    power_consumption: f64::from(battery.energy_rate().get::<watt>()),
                    health_percent: f64::from(battery.state_of_health().get::<percent>()),
                    state: match battery.state() {
                        State::Unknown => BatteryState::Unknown,
                        State::Charging => BatteryState::Charging {
                            time_to_full: {
                                let optional_time = battery.time_to_full();
                                optional_time.map(|time| f64::from(time.get::<second>()) as u32)
                            },
                        },
                        State::Discharging => BatteryState::Discharging {
                            time_to_empty: {
                                let optional_time = battery.time_to_empty();
                                optional_time.map(|time| f64::from(time.get::<second>()) as u32)
                            },
                        },
                        State::Empty => BatteryState::Empty,
                        State::Full => BatteryState::Full,
                    },
                })
            } else {
                None
            }
        })
        .collect::<Vec<_>>()
}
