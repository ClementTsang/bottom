cfg_if::cfg_if! {
    if #[cfg(target_os = "freebsd")] {
        mod freebsd;
        pub(crate) use self::freebsd::*;
    } else if #[cfg(target_os = "windows")] {
        mod windows;
        pub(crate) use self::windows::*;
    } else if #[cfg(target_family = "unix")] {
        mod unix;
        pub(crate) use self::unix::*;
    }
}
