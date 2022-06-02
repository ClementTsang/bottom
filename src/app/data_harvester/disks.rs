//! Data collection for disks (IO, usage, space, etc.).
//!
//! For Linux, macOS, and Windows, this is handled by heim.

cfg_if::cfg_if! {
    if #[cfg(any(target_os = "linux", target_os = "macos", target_os = "windows"))] {
        pub mod heim;
        pub use self::heim::*;
    }
}
