//! macOS and Windows-specific things for Heim disk data collection.

use heim::disk::Partition;
use std::ffi::OsString;

pub fn get_device_name(partition: &Partition) -> String {
    if let Some(device) = partition.device() {
        device
            .to_string_lossy()
            .unwrap_or_else(|_| "Name Unavailable".to_string())
    } else {
        "Name Unavailable".to_string()
    }
}
