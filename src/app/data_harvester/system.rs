use sysinfo::{LoadAvg, System, SystemExt};

pub fn get_load_average(sys: &System) -> LoadAvg {
    sys.get_load_average()
}

/// In seconds.
pub fn get_uptime(sys: &System) -> u64 {
    sys.get_uptime()
}
