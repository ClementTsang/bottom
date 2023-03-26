//! Data collection about disks (e.g. I/O, usage, space).

use std::collections::HashMap;

cfg_if::cfg_if! {
    if #[cfg(target_os = "freebsd")] {
        pub mod freebsd;
        pub use self::freebsd::*;
    } else if #[cfg(target_os = "windows")] {
        pub mod windows;
        pub use self::windows::*;
    } else if #[cfg(target_os = "linux")] {
        pub mod unix;
        pub use self::unix::*;
    } else if #[cfg(target_os = "macos")] {
        pub mod unix;
        pub use self::unix::*;
    }
    // TODO: Add dummy impls here for other OSes?
}

#[derive(Debug, Clone, Default)]
pub struct DiskHarvest {
    pub name: String,
    pub mount_point: String,

    // TODO: Maybe unify all these?
    pub free_space: u64,
    pub used_space: u64,
    pub total_space: u64,
}

#[derive(Clone, Debug)]
pub struct IoData {
    pub read_bytes: u64,
    pub write_bytes: u64,
}

pub type IoHarvest = HashMap<String, Option<IoData>>;
