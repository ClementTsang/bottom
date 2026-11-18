#[cfg(feature = "nvidia")]
pub mod nvidia;

cfg_select! {
    all(target_os = "linux", feature = "gpu") => {
        pub mod intel;
        pub mod amd;
    }
    all(not(target_os = "linux"), feature = "gpu") => {
        pub mod sysinfo_backend;
    }
    _ => {}
}
