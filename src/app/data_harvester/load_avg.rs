use sysinfo::{System, SystemExt};

pub type LoadAvgHarvest = [f64; 3];

pub async fn get_load_avg() -> crate::error::Result<LoadAvgHarvest> {
    let s = System::new_all();
    let load_avg = s.get_load_average();
    Ok([load_avg.one, load_avg.five, load_avg.fifteen])
}
