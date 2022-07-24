//! Data collection for disks (IO, usage, space, etc.).
//!
//! For Linux, macOS, and Windows, this is handled by heim. For FreeBSD there is a custom
//! implementation.

cfg_if::cfg_if! {
    if #[cfg(any(target_os = "linux", target_os = "macos", target_os = "windows"))] {
        pub mod heim;
        pub use self::heim::*;
    } else if #[cfg(target_os = "freebsd")] {
        pub mod freebsd;
        pub use self::freebsd::*;
    }
}

#[derive(Debug, Clone, Default)]
pub struct DiskHarvest {
    pub name: String,
    pub mount_point: String,
    pub free_space: Option<u64>,
    pub used_space: Option<u64>,
    pub total_space: Option<u64>,
}

#[derive(Clone, Debug)]
pub struct IoData {
    pub read_bytes: u64,
    pub write_bytes: u64,
}

pub type IoHarvest = std::collections::HashMap<String, Option<IoData>>;
