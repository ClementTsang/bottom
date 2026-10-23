#[cfg(feature = "nvidia")]
pub mod nvidia;

cfg_select! {
    all(target_os = "linux", feature = "gpu") =>{
        pub mod intel;
        pub mod amd;
    }
    all(target_os = "windows", feature = "gpu") => {
        pub mod amd_windows;
    }
    _ => {}
}
