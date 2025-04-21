//! Data collection for temperature metrics.
//!
//! For Linux, this is handled by custom code.
//! For everything else, this is handled by sysinfo.

cfg_if::cfg_if! {
    if #[cfg(target_os = "linux")] {
        pub mod linux;
        pub use self::linux::*;
    } else if #[cfg(any(target_os = "freebsd", target_os = "macos", target_os = "windows", target_os = "android", target_os = "ios"))] {
        pub mod sysinfo;
        pub use self::sysinfo::*;
    }
}

#[derive(Default, Debug, Clone)]
pub struct TempSensorData {
    /// The name of the sensor.
    pub name: String,

    /// The temperature in Celsius.
    pub temperature: Option<f32>,
}
