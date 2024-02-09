use serde::Deserialize;

use crate::args::GpuArgs;

use super::DefaultConfig;

#[derive(Clone, Debug, Default, Deserialize)]
pub(crate) struct GpuConfig {
    #[serde(flatten)]
    pub(crate) args: GpuArgs,
}

impl GpuConfig {
    pub(crate) fn enabled(&self) -> bool {
        self.args.enable_gpu.unwrap_or(false)
    }
}

impl DefaultConfig for GpuConfig {
    fn default_config() -> String {
        todo!()
    }
}
