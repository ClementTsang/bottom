use serde::Deserialize;

use crate::args::GpuArgs;

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
