//! Data collection for CPU usage and load average.
//!
//! For CPU usage, Linux, macOS, and Windows are handled by Heim.
//!
//! For load average, macOS and Linux are supported through Heim.

cfg_if::cfg_if! {
    if #[cfg(any(target_os = "linux", target_os = "macos", target_os = "windows"))] {
        pub mod heim;
        pub use self::heim::*;
    }
}

pub type LoadAvgHarvest = [f32; 3];
