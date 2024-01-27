use serde::Deserialize;

use crate::args::GpuArgs;

#[cfg(feature = "gpu")]
#[derive(Clone, Debug, Default, Deserialize)]
pub(crate) struct GpuOptions {
    #[serde(flatten)]
    pub(crate) args: GpuArgs,
}
