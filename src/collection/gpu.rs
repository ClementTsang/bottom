#[cfg(feature = "nvidia")]
pub mod nvidia;

#[cfg(all(target_os = "linux", feature = "gpu"))]
pub mod amd;
