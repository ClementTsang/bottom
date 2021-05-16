//! Data collection for network usage/IO.
//!
//! For Linux and macOS, this is handled by Heim.
//! For Windows, this is handled by sysinfo.

cfg_if::cfg_if! {
    if #[cfg(any(target_os = "linux", target_os = "macos"))] {
        pub mod heim;
        pub use self::heim::*;
    } else if #[cfg(target_os = "windows")] {
        pub mod sysinfo;
        pub use self::sysinfo::*;
    }
}

#[derive(Default, Clone, Debug)]
/// All units in bits.
pub struct NetworkHarvest {
    pub rx: u64,
    pub tx: u64,
    pub total_rx: u64,
    pub total_tx: u64,
}

impl NetworkHarvest {
    pub fn first_run_cleanup(&mut self) {
        self.rx = 0;
        self.tx = 0;
    }
}
