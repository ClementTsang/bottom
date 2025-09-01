use std::{borrow::Cow, fs, os::unix::ffi::OsStrExt, path::Path};

use libc::PATH_MAX;

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

/// A custom implementation to read a symlink while allowing for buffer reuse.
///
/// If successful, then a [`Cow`] will be returned referencing the contents of `buffer`.
pub(crate) fn read_link<'a>(path: &Path, buffer: &'a mut Vec<u8>) -> std::io::Result<Cow<'a, str>> {
    let c_path = std::ffi::CString::new(path.as_os_str().as_bytes())?;

    if buffer.len() < PATH_MAX as usize {
        buffer.resize(PATH_MAX as usize, 0);
    }

    // SAFETY: this is a libc API; we must check the length which we do below.
    let len = unsafe {
        libc::readlink(
            c_path.as_ptr(),
            buffer.as_mut_ptr() as *mut libc::c_char,
            buffer.len(),
        )
    };

    if len < 0 {
        return Err(std::io::Error::last_os_error());
    }
    Ok(String::from_utf8_lossy(&buffer[..len as usize]))
}
