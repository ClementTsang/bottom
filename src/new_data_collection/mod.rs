//! Module that just re-exports the right data collector for a given platform.

pub mod error;

mod collectors {
    pub mod common;

    cfg_if::cfg_if! {
        if #[cfg(target_os = "linux")] {
            pub mod linux;
            pub use linux::LinuxDataCollector as DataCollectorImpl;
        // } else if #[cfg(target_os = "macos")] {
        //     pub mod macos;
        //     pub use macos::MacOsDataCollector as DataCollectorImpl;
        // } else if #[cfg(target_os = "windows")] {
        //     pub mod windows;
        //     pub use windows::WindowsDataCollector as DataCollectorImpl;
        // } else if #[cfg(target_os = "freebsd")] {
        //     pub mod freebsd;
        //     pub use freebsd::FreeBsdDataCollector as DataCollectorImpl;
        } else {
            pub mod fallback;
            pub use fallback::FallbackDataCollector as DataCollectorImpl;
        }
    }
}

pub mod sources;
