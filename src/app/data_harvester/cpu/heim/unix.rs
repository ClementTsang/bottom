//! Unix-specific functions regarding CPU usage.

use crate::app::data_harvester::cpu::LoadAvgHarvest;

pub async fn get_load_avg() -> crate::error::Result<LoadAvgHarvest> {
    let (one, five, fifteen) = heim::cpu::os::unix::loadavg().await?;

    Ok([
        one.get::<heim::units::ratio::ratio>(),
        five.get::<heim::units::ratio::ratio>(),
        fifteen.get::<heim::units::ratio::ratio>(),
    ])
}
