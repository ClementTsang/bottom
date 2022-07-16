//! Data collection for memory.
//!
//! For Linux, macOS, and Windows, this is handled by Heim. On FreeBSD it is handled by sysinfo.

cfg_if::cfg_if! {
    if #[cfg(any(target_os = "freebsd", target_os = "linux", target_os = "macos", target_os = "windows"))] {
        pub mod general;
        pub use self::general::*;
    }
}
