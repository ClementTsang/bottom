#[cfg(not(target_os = "linux"))]
use sysinfo::{System, SystemExt};

pub type LoadAvgHarvest = [f32; 3];

#[cfg(target_family = "unix")]
pub async fn get_load_avg() -> crate::error::Result<LoadAvgHarvest> {
    let (one, five, fifteen) = heim::cpu::os::unix::loadavg().await?;

    Ok([
        one.get::<heim::units::ratio::ratio>(),
        five.get::<heim::units::ratio::ratio>(),
        fifteen.get::<heim::units::ratio::ratio>(),
    ])
}
