cfg_if::cfg_if! {
    if #[cfg(any(target_os = "linux", target_os = "macos", target_os = "windows"))] {
        pub mod heim;
        pub use self::heim::*;
    } else if #[cfg(target_os = "freebsd")] {
        pub mod sysinfo;
        pub use self::sysinfo::*;
    }
}

#[derive(Debug, Clone, Default)]
pub struct MemHarvest {
    pub mem_total_in_kib: u64,
    pub mem_used_in_kib: u64,
    pub use_percent: Option<f64>,
}
