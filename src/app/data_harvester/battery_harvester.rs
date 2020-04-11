use battery::{Battery, Manager};

#[derive(Default, Debug, Clone)]
pub struct BatteryHarvest {
    charge_percent: u64,
    is_charging: bool,
    secs_until_full: u64,
    secs_until_empty: u64,
    power_consumption_rate: f64,
    voltage: u64,
}

pub fn refresh_batteries(manager: &Manager, batteries: &mut [Battery]) {
    for battery in batteries {
        if manager.refresh(battery).is_ok() {
            debug!("Battery: {:?}", battery);
        }
    }
}
