//! Windows bindings to get disk I/O counters.

use std::{
    ffi::OsString,
    io, mem,
    os::windows::prelude::{OsStrExt, OsStringExt},
    path::{Path, PathBuf},
};

use anyhow::bail;
use windows::Win32::{
    Foundation::{self, CloseHandle, HANDLE},
    Storage::FileSystem::{
        CreateFileW, FILE_FLAGS_AND_ATTRIBUTES, FILE_SHARE_READ, FILE_SHARE_WRITE,
        FindFirstVolumeW, FindNextVolumeW, FindVolumeClose, GetVolumeNameForVolumeMountPointW,
        OPEN_EXISTING,
    },
    System::{
        IO::DeviceIoControl,
        Ioctl::{DISK_PERFORMANCE, IOCTL_DISK_PERFORMANCE},
    },
};

/// Returns the I/O for a given volume.
///
/// Based on [psutil's implementation](https://github.com/giampaolo/psutil/blob/52fe5517f716dedf9c9918e56325e49a49146130/psutil/arch/windows/disk.c#L78-L83)
/// and [heim's implementation](https://github.com/heim-rs/heim/blob/master/heim-disk/src/sys/windows/bindings/perf.rs).
fn volume_io(volume: &Path) -> anyhow::Result<DISK_PERFORMANCE> {
    if volume.is_file() {
        // We assume the volume is a directory, so bail ASAP if it isn't.
        bail!("Expects a directory to be passed in.");
    }

    let volume = {
        let mut wide_path = volume.as_os_str().encode_wide().collect::<Vec<_>>();

        // We replace the trailing backslash and replace it with a \0.
        wide_path.pop();
        wide_path.push(0x0000);

        wide_path
    };

    // SAFETY: API call, arguments should be correct. We must also check after the
    // call to ensure it is valid.
    let h_device = unsafe {
        CreateFileW(
            windows::core::PCWSTR(volume.as_ptr()),
            0,
            FILE_SHARE_READ | FILE_SHARE_WRITE,
            None,
            OPEN_EXISTING,
            FILE_FLAGS_AND_ATTRIBUTES(0),
            Some(Foundation::HANDLE::default()),
        )?
    };

    if h_device.is_invalid() {
        bail!("Invalid handle value: {:?}", io::Error::last_os_error());
    }

    let mut disk_performance = DISK_PERFORMANCE::default();
    let mut bytes_returned = 0;

    // SAFETY: This should be safe, we'll manually check the results and the
    // arguments should be valid.
    let ret = unsafe {
        DeviceIoControl(
            h_device,
            IOCTL_DISK_PERFORMANCE,
            None,
            0,
            Some(&mut disk_performance as *mut _ as _),
            mem::size_of::<DISK_PERFORMANCE>() as u32,
            Some(&mut bytes_returned),
            None,
        )
    };

    // SAFETY: This should be safe, we will check the result as well.
    let handle_result = unsafe { CloseHandle(h_device) };
    if let Err(err) = handle_result {
        bail!("Handle error: {err:?}");
    }

    if let Err(err) = ret {
        bail!("Device I/O error: {err:?}");
    } else {
        Ok(disk_performance)
    }
}

fn current_volume(buffer: &[u16]) -> PathBuf {
    let first_null = buffer.iter().position(|byte| *byte == 0x00).unwrap_or(0);
    let path_string = OsString::from_wide(&buffer[..first_null]);

    PathBuf::from(path_string)
}

fn close_find_handle(handle: HANDLE) -> anyhow::Result<()> {
    // Clean up the handle.
    // SAFETY: This should be safe, we will check the result as well.
    let res = unsafe { FindVolumeClose(handle) };
    Ok(res?)
}

/// Returns the I/O for all volumes.
///
/// Based on [psutil's implementation](https://github.com/giampaolo/psutil/blob/52fe5517f716dedf9c9918e56325e49a49146130/psutil/arch/windows/disk.c#L78-L83)
/// and [heim's implementation](https://github.com/heim-rs/heim/blob/master/heim-disk/src/sys/windows/bindings/perf.rs).
pub(crate) fn all_volume_io() -> anyhow::Result<Vec<anyhow::Result<(DISK_PERFORMANCE, String)>>> {
    const ERROR_NO_MORE_FILES: i32 = Foundation::ERROR_NO_MORE_FILES.0 as i32;
    let mut ret = vec![];
    let mut buffer = [0_u16; Foundation::MAX_PATH as usize];

    // Get the first volume and add the stats needed.
    // SAFETY: We must verify the handle is correct. If no volume is found, it will
    // be set to `INVALID_HANDLE_VALUE`.
    let handle = unsafe { FindFirstVolumeW(&mut buffer) }?;
    if handle.is_invalid() {
        bail!("Invalid handle value: {:?}", io::Error::last_os_error());
    }

    {
        let volume = current_volume(&buffer);
        ret.push(volume_io(&volume).map(|res| (res, volume.to_string_lossy().to_string())));
    }

    // Now iterate until there are no more volumes.
    while unsafe { FindNextVolumeW(handle, &mut buffer) }.is_ok() {
        let volume = current_volume(&buffer);
        ret.push(volume_io(&volume).map(|res| (res, volume.to_string_lossy().to_string())));
    }

    let err = io::Error::last_os_error();
    match err.raw_os_error() {
        Some(ERROR_NO_MORE_FILES) => {
            // Iteration completed successfully, continue on.
        }
        _ => {
            // Some error occured.
            close_find_handle(handle)?;
            bail!("Error while iterating over volumes: {err:?}");
        }
    }

    close_find_handle(handle)?;

    Ok(ret)
}

/// Returns the volume name from a mount name if possible.
pub(crate) fn volume_name_from_mount(mount: &str) -> anyhow::Result<String> {
    // According to winapi docs 50 is a reasonable length to accomodate the volume
    // path https://docs.microsoft.com/en-us/windows/win32/api/fileapi/nf-fileapi-getvolumenameforvolumemountpointw
    const VOLUME_MAX_LEN: usize = 50;

    let mount = {
        let mount_path = Path::new(mount);
        let mut wide_path = mount_path.as_os_str().encode_wide().collect::<Vec<_>>();

        // Always push on a \0 character, without this it will occasionally break.
        wide_path.push(0x0000);

        wide_path
    };
    let mut buffer = [0_u16; VOLUME_MAX_LEN];

    // SAFETY: API call, we must check the result for validating safety.
    let result = unsafe {
        GetVolumeNameForVolumeMountPointW(windows::core::PCWSTR(mount.as_ptr()), &mut buffer)
    };

    if let Err(err) = result {
        bail!("Could not get volume name for mount point: {err:?}");
    } else {
        Ok(current_volume(&buffer).to_string_lossy().to_string())
    }
}
