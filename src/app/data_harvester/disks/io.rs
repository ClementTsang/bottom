cfg_if::cfg_if! {
    if #[cfg(target_os = "freebsd")] {
        pub mod freebsd;
        pub use self::freebsd::*;
    } else if #[cfg(target_os = "windows")] {
        pub mod windows;
        pub use self::windows::*;
    } else if #[cfg(target_os = "linux")] {
        pub mod linux;
        pub use self::linux::*;
    } else if #[cfg(target_os = "macos")] {
        pub mod macos;
        pub use self::macos::*;
    }
}
