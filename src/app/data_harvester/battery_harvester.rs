use battery::{
    units::{power::watt, ratio::percent, time::second, Time},
    Battery, Manager,
};

#[derive(Debug, Clone)]
pub struct BatteryHarvest {
    pub charge_percent: u64,
    pub secs_until_full: Option<i64>,
    pub secs_until_empty: Option<i64>,
    pub power_consumption_rate_watts: f64,
}

fn convert_optional_time_to_optional_seconds(optional_time: Option<Time>) -> Option<i64> {
    if let Some(time) = optional_time {
        Some(f64::from(time.get::<second>()) as i64)
    } else {
        None
    }
}

pub fn refresh_batteries(manager: &Manager, batteries: &mut [Battery]) -> Vec<BatteryHarvest> {
    batteries
        .iter_mut()
        .filter_map(|battery| {
            if manager.refresh(battery).is_ok() {
                Some(BatteryHarvest {
                    secs_until_full: convert_optional_time_to_optional_seconds(
                        battery.time_to_full(),
                    ),
                    secs_until_empty: convert_optional_time_to_optional_seconds(
                        battery.time_to_empty(),
                    ),
                    charge_percent: f64::from(battery.state_of_charge().get::<percent>()) as u64,
                    power_consumption_rate_watts: f64::from(battery.energy_rate().get::<watt>()),
                })
            } else {
                None
            }
        })
        .collect::<Vec<_>>()
}
