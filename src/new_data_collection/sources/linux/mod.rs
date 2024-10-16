mod processes;
mod temperature;

pub(crate) use processes::*;
pub(crate) use temperature::*;

// For now we only use a Linux-specific implementation for zfs ARC usage.
cfg_if::cfg_if! {
    if #[cfg(feature = "zfs")] {
        mod memory;
        pub(crate) use memory::*;
    }
}
