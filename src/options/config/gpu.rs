use serde::Deserialize;

#[derive(Clone, Debug, Default, Deserialize)]
pub(crate) struct GpuOptions {
    pub(crate) enable_gpu: Option<bool>, // TODO: Enable by default instead?
}
