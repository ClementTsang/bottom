use nvml_wrapper::{error::NvmlError, NVML};
use once_cell::sync::Lazy;
pub static NVML_DATA: Lazy<Result<NVML, NvmlError>> = Lazy::new(NVML::init);
