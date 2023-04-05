use std::{
    ffi::{CStr, CString},
    os::unix::prelude::OsStrExt,
    path::{Path, PathBuf},
    str::FromStr,
};

use anyhow::bail;

use super::bindings;
use crate::app::data_harvester::disks::system::{FileSystem, Usage};

pub(crate) struct Partition {
    device: String,
    mount_point: PathBuf,
    fs_type: FileSystem,
}

impl Partition {
    /// Returns the device name, if there is one.
    #[inline]
    pub fn device(&self) -> &str {
        self.device.as_str()
    }

    /// Returns the mount point for this partition.
    #[inline]
    pub fn mount_point(&self) -> &Path {
        self.mount_point.as_path()
    }

    /// Returns the [`FileSystem`] of this partition.
    #[inline]
    pub fn fs_type(&self) -> &FileSystem {
        &self.fs_type
    }

    /// Returns the usage stats for this partition.
    pub fn usage(&self) -> anyhow::Result<Usage> {
        let path = CString::new(self.mount_point().as_os_str().as_bytes())?;
        let mut vfs = std::mem::MaybeUninit::<libc::statvfs>::uninit();

        let result = unsafe { libc::statvfs(path.as_ptr(), vfs.as_mut_ptr()) };

        if result == 0 {
            Ok(Usage::new(unsafe { vfs.assume_init() }))
        } else {
            bail!("statvfs failed to get the disk usage for disk {path:?}")
        }
    }

    /// Returns the device name.
    #[inline]
    pub fn get_device_name(&self) -> String {
        self.device.clone()
    }
}

fn partitions_iter() -> anyhow::Result<impl Iterator<Item = Partition>> {
    let mounts = bindings::mounts()?;

    fn ptr_to_str<'a>(ptr: *const i8) -> std::borrow::Cow<'a, str> {
        unsafe { CStr::from_ptr(ptr) }.to_string_lossy()
    }

    Ok(mounts.into_iter().map(|stat| {
        let device = ptr_to_str(stat.f_mntfromname.as_ptr()).to_string();

        let fs_type = {
            let fs_type_str = ptr_to_str(stat.f_fstypename.as_ptr());

            FileSystem::from_str(&fs_type_str).unwrap_or(FileSystem::Other(fs_type_str.to_string()))
        };

        let mount_point = {
            let path_str = ptr_to_str(stat.f_mntonname.as_ptr()).to_string();

            PathBuf::from(path_str)
        };

        Partition {
            device,
            mount_point,
            fs_type,
        }
    }))
}

#[allow(dead_code)]
/// Returns a [`Vec`] containing all partitions.
pub(crate) fn partitions() -> anyhow::Result<Vec<Partition>> {
    partitions_iter().map(|iter| iter.collect())
}

/// Returns a [`Vec`] containing all *physical* partitions. This is defined by
/// [`FileSystem::is_physical()`].
pub(crate) fn physical_partitions() -> anyhow::Result<Vec<Partition>> {
    partitions_iter().map(|iter| {
        iter.filter(|partition| partition.fs_type().is_physical())
            .collect()
    })
}
