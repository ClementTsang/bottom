//! Code pertaining to memory data retrieval.

#[derive(Debug)]
pub(crate) struct MemData {
    pub used_bytes: u64,
    pub total_bytes: u64,
}

#[derive(Debug)]
pub(crate) struct MemHarvest {
    pub memory: MemData,
    pub swap: MemData,

    #[cfg(not(target_os = "windows"))]
    pub cache: MemData,

    #[cfg(feature = "zfs")]
    pub arc: MemData,

    #[cfg(feature = "gpu")]
    pub gpu: Vec<(String, MemData)>,
}
