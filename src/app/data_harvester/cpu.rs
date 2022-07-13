//! Data collection for CPU usage and load average.
//!
//! For CPU usage, Linux, macOS, and Windows are handled by Heim.
//!
//! For load average, macOS and Linux are supported through Heim.

cfg_if::cfg_if! {
    if #[cfg(any(target_os = "linux", target_os = "macos", target_os = "windows"))] {
        pub mod heim;
        pub use self::heim::*;
    } else if #[cfg(target_os = "freebsd")] {
        pub mod sysinfo;
        pub use self::sysinfo::*;
    }
}

pub type LoadAvgHarvest = [f32; 3];

#[derive(Default, Debug, Clone)]
pub struct CpuData {
    pub cpu_prefix: String,
    pub cpu_count: Option<usize>,
    pub cpu_usage: f64,
}

pub type CpuHarvest = Vec<CpuData>;

pub type PastCpuWork = f64;
pub type PastCpuTotal = f64;
