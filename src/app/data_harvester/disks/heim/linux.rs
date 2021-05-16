//! Linux-specific things for Heim disk data collection.

use heim::disk::Partition;

pub fn get_device_name(partition: &Partition) -> String {
    if let Some(device) = partition.device() {
        // See if this disk is actually mounted elsewhere on Linux...
        // This is a workaround to properly map I/O in some cases (i.e. disk encryption), see
        // https://github.com/ClementTsang/bottom/issues/419
        if let Ok(path) = std::fs::read_link(device) {
            if path.is_absolute() {
                path.into_os_string()
            } else {
                let mut combined_path = std::path::PathBuf::new();
                combined_path.push(device);
                combined_path.pop(); // Pop the current file...
                combined_path.push(path);

                if let Ok(canon_path) = std::fs::canonicalize(combined_path) {
                    // Resolve the local path into an absolute one...
                    canon_path.into_os_string()
                } else {
                    device.to_os_string()
                }
            }
        } else {
            device.to_os_string()
        }
        .into_string()
        .unwrap_or_else(|_| "Name Unavailable".to_string())
    } else {
        "Name Unavailable".to_string()
    }
}
