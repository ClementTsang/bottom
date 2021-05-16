//! macOS and Windows-specific things for Heim disk data collection.

use heim::disk::Partition;

pub fn get_device_name(partition: &Partition) -> String {
    if let Some(device) = partition.device() {
        device
            .to_os_string()
            .into_string()
            .unwrap_or_else(|_| "Name Unavailable".to_string())
    } else {
        "Name Unavailable".to_string()
    }
}
