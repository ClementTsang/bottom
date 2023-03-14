//! Data collection about disks (e.g. I/O, usage, space).

use std::collections::HashMap;

pub mod io;
pub mod usage;

#[derive(Debug, Clone, Default)]
pub struct DiskHarvest {
    pub name: String,
    pub mount_point: String,

    // TODO: Maybe unify all these?
    pub free_space: u64,
    pub used_space: u64,
    pub total_space: u64,
}

#[derive(Clone, Debug)]
pub struct IoData {
    pub read_bytes: u64,
    pub write_bytes: u64,
}

pub type IoHarvest = HashMap<String, Option<IoData>>;
