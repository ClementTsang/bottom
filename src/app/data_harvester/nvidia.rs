use nvml_wrapper::{error::NvmlError, Nvml};
use once_cell::sync::Lazy;

pub static NVML_DATA: Lazy<Result<Nvml, NvmlError>> = Lazy::new(Nvml::init);
