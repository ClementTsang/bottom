pub mod common;
pub mod linux;
// pub mod macos;
#[cfg(feature = "gpu")]
pub mod nvidia;
pub mod starship;
pub mod sysinfo;
pub mod unix;
// pub mod windows;

pub use common::*;

cfg_if::cfg_if! {
    if #[cfg(target_family = "unix")] {
        pub use unix::processes::Pid as Pid;
    }
    // else if #[cfg(target_family = "windows")] {
    //     pub use windows::processes::Pid as Pid;
    // }
}
