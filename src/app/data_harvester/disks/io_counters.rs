use std::ffi::OsStr;

#[derive(Debug, Default)]
pub struct IoCounters {
    name: String,
    read_bytes: u64,
    write_bytes: u64,
}

impl IoCounters {
    pub fn new(name: String, read_bytes: u64, write_bytes: u64) -> Self {
        Self {
            name,
            read_bytes,
            write_bytes,
        }
    }

    pub(crate) fn device_name(&self) -> &OsStr {
        OsStr::new(&self.name)
    }

    pub(crate) fn read_bytes(&self) -> u64 {
        self.read_bytes
    }

    pub(crate) fn write_bytes(&self) -> u64 {
        self.write_bytes
    }
}
