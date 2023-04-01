//! Implementation based on [heim's](https://github.com/heim-rs/heim)
//! Unix disk usage.

use std::{
    ffi::CString,
    fs::File,
    io::{self, BufRead, BufReader},
    mem,
    path::{Path, PathBuf},
    str::FromStr,
};

use anyhow::bail;

use crate::app::data_harvester::disks::unix::{FileSystem, Usage};

/// Representation of partition details. Based on [`heim`](https://github.com/heim-rs/heim/tree/master).
pub(crate) struct Partition {
    device: Option<String>,
    mount_point: PathBuf,
    fs_type: FileSystem,
}

impl Partition {
    /// Returns the device name, if there is one.
    pub fn device(&self) -> Option<&str> {
        self.device.as_deref()
    }

    /// Returns the mount point for this partition.
    pub fn mount_point(&self) -> &Path {
        self.mount_point.as_path()
    }

    /// Returns the [`FileSystem`] of this partition.
    pub fn fs_type(&self) -> &FileSystem {
        &self.fs_type
    }

    /// Returns the device name for the partition.
    pub fn get_device_name(&self) -> String {
        if let Some(device) = self.device() {
            // See if this disk is actually mounted elsewhere on Linux. This is a workaround properly map I/O
            // in some cases (i.e. disk encryption, https://github.com/ClementTsang/bottom/issues/419).
            if let Ok(path) = std::fs::read_link(device) {
                if path.is_absolute() {
                    path.into_os_string()
                        .into_string()
                        .unwrap_or_else(|_| "Name unavailable".to_string())
                } else {
                    let mut combined_path = std::path::PathBuf::new();
                    combined_path.push(device);
                    combined_path.pop(); // Pop the current file...
                    combined_path.push(path);

                    if let Ok(canon_path) = std::fs::canonicalize(combined_path) {
                        // Resolve the local path into an absolute one...
                        canon_path
                            .into_os_string()
                            .into_string()
                            .unwrap_or_else(|_| "Name unavailable".to_string())
                    } else {
                        device.to_owned()
                    }
                }
            } else {
                device.to_owned()
            }
        } else {
            "Name unavailable".to_string()
        }
    }

    /// Returns the usage stats for this partition.
    pub fn usage(&self) -> anyhow::Result<Usage> {
        let path = self
            .mount_point
            .to_str()
            .ok_or_else(|| io::Error::from(io::ErrorKind::InvalidInput))
            .and_then(|string| {
                CString::new(string).map_err(|_| io::Error::from(io::ErrorKind::InvalidInput))
            })
            .map_err(|e| anyhow::anyhow!("invalid path: {e:?}"))?;

        let mut vfs = mem::MaybeUninit::<libc::statvfs>::uninit();
        let result = unsafe { libc::statvfs(path.as_ptr(), vfs.as_mut_ptr()) };

        if result == 0 {
            let vfs = unsafe { vfs.assume_init() };
            Ok(Usage::new(vfs))
        } else {
            Err(anyhow::anyhow!(
                "statvfs had an issue getting info from {path:?}"
            ))
        }
    }
}

impl FromStr for Partition {
    type Err = anyhow::Error;

    fn from_str(line: &str) -> anyhow::Result<Partition> {
        // Example: `/dev/sda3 /home ext4 rw,relatime,data=ordered 0 0`
        let mut parts = line.splitn(5, ' ');

        let device = match parts.next() {
            Some(device) if device == "none" => None,
            Some(device) => Some(device.to_string()),
            None => {
                bail!("missing device");
            }
        };
        let mount_point = match parts.next() {
            Some(point) => PathBuf::from(point),
            None => {
                bail!("missing mount point");
            }
        };
        let fs_type = match parts.next() {
            Some(fs) => FileSystem::from_str(fs)?,
            _ => {
                bail!("missing filesystem type");
            }
        };

        // let options = match parts.next() {
        //     Some(opts) => opts.to_string(),
        //     None => {
        //         bail!("missing options");
        //     }
        // };

        Ok(Partition {
            device,
            mount_point,
            fs_type,
        })
    }
}

#[allow(dead_code)]
/// Returns all partitions.
pub(crate) fn partitions() -> anyhow::Result<Vec<Partition>> {
    const PROC_MOUNTS: &str = "/proc/mounts";

    let mounts = BufReader::new(File::open(PROC_MOUNTS)?).lines();
    Ok(mounts
        .filter_map(|line| match line {
            Ok(line) => Partition::from_str(&line).ok(),
            Err(_) => None,
        })
        .collect())
}

/// Returns all physical partitions.
pub(crate) fn partitions_physical() -> anyhow::Result<Vec<Partition>> {
    const PROC_MOUNTS: &str = "/proc/mounts";

    let mounts = BufReader::new(File::open(PROC_MOUNTS)?).lines();
    Ok(mounts
        .filter_map(|line| match line {
            Ok(line) => Partition::from_str(&line).ok(),
            Err(_) => None,
        })
        .filter(|partition| partition.fs_type().is_physical())
        .collect())
}
