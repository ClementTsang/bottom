//! Data collection for batteries.
//!
//! For Linux, macOS, Windows, FreeBSD, Dragonfly, and iOS, this is handled by
//! the battery crate.

cfg_if::cfg_if! {
    if #[cfg(any(
        target_os = "windows",
        target_os = "macos",
        target_os = "linux",
        target_os = "freebsd",
        target_os = "dragonfly",
        target_os = "ios",
    ))] {
        pub mod battery;
        pub use self::battery::*;
    }
}
