//! Re-exports all of the sources.

cfg_if::cfg_if! {
    if #[cfg(any(
        target_os = "windows",
        target_os = "macos",
        target_os = "linux",
        target_os = "freebsd",
        target_os = "dragonfly",
        target_os = "ios",
    ))] {
        pub mod starship_battery;
    }
}
