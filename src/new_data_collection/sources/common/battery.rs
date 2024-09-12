//! Common code for retrieving battery data.

#[derive(Debug, Clone)]
pub enum State {
    Unknown,
    Charging,
    Discharging,
    Empty,
    Full,
}

#[derive(Debug, Clone)]
pub struct BatteryHarvest {
    pub charge_percent: f64,
    pub secs_until_full: Option<i64>,
    pub secs_until_empty: Option<i64>,
    pub power_consumption_rate_watts: f64,
    pub health_percent: f64,
    pub state: State,
}
