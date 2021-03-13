#[cfg(not(target_os = "linux"))]
use sysinfo::{System, SystemExt};

pub type LoadAvgHarvest = [f32; 3];

#[cfg(not(target_os = "linux"))]
pub fn get_load_avg(sys: &System) -> LoadAvgHarvest {
    let load_avg = sys.get_load_average();
    [
        load_avg.one as f32,
        load_avg.five as f32,
        load_avg.fifteen as f32,
    ]
}

#[cfg(target_os = "linux")]
pub async fn get_load_avg() -> crate::error::Result<LoadAvgHarvest> {
    let (one, five, fifteen) = heim::cpu::os::unix::loadavg().await?;

    Ok([
        one.get::<heim::units::ratio::ratio>(),
        five.get::<heim::units::ratio::ratio>(),
        fifteen.get::<heim::units::ratio::ratio>(),
    ])
}
