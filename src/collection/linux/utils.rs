use std::{fs, path::Path};

/// Whether the temperature should *actually* be read during enumeration.
/// Will return false if the state is not D0/unknown, or if it does not support
/// `device/power_state`.
///
/// `path` is a path to the device itself (e.g. `/sys/class/hwmon/hwmon1/device`).
#[inline]
pub fn is_device_awake(device: &Path) -> bool {
    // Whether the temperature should *actually* be read during enumeration.
    // Set to false if the device is in ACPI D3cold.
    // Documented at https://www.kernel.org/doc/Documentation/ABI/testing/sysfs-devices-power_state
    let power_state = device.join("power_state");
    if power_state.exists() {
        if let Ok(state) = fs::read_to_string(power_state) {
            let state = state.trim();
            // The zenpower3 kernel module (incorrectly?) reports "unknown", causing this
            // check to fail and temperatures to appear as zero instead of
            // having the file not exist.
            //
            // Their self-hosted git instance has disabled sign up, so this bug cant be
            // reported either.
            state == "D0" || state == "unknown"
        } else {
            true
        }
    } else {
        true
    }
}
