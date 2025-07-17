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

use crate::collection::disks::unix::{FileSystem, Usage};

/// Representation of partition details. Based on [`heim`](https://github.com/heim-rs/heim/tree/master).
pub(crate) struct Partition {
    device: Option<String>,
    mount_point: PathBuf,
    fs_type: FileSystem,
}

impl Partition {
    /// Returns the device name, if there is one.
    #[inline]
    pub fn device(&self) -> Option<&str> {
        self.device.as_deref()
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

    /// Returns the device name for the partition.
    pub fn get_device_name(&self) -> String {
        if let Some(device) = self.device() {
            // See if this disk is actually mounted elsewhere on Linux. This is a workaround
            // properly map I/O in some cases (i.e. disk encryption, https://github.com/ClementTsang/bottom/issues/419).
            if let Ok(path) = std::fs::read_link(device) {
                if path.is_absolute() {
                    path.into_os_string()
                        .into_string()
                        .unwrap_or_else(|_| "Name Unavailable".to_string())
                } else {
                    let mut combined_path = PathBuf::new();
                    combined_path.push(device);
                    combined_path.pop(); // Pop the current file...
                    combined_path.push(path);

                    if let Ok(canon_path) = std::fs::canonicalize(combined_path) {
                        // Resolve the local path into an absolute one...
                        canon_path
                            .into_os_string()
                            .into_string()
                            .unwrap_or_else(|_| "Name Unavailable".to_string())
                    } else {
                        device.to_owned()
                    }
                }
            } else {
                device.to_owned()
            }
        } else {
            "Name Unavailable".to_string()
        }
    }

    /// Returns the usage stats for this partition.
    pub fn usage(&self) -> anyhow::Result<Usage> {
        // TODO: This might be unoptimal.
        let path = self
            .mount_point
            .to_str()
            .ok_or_else(|| io::Error::from(io::ErrorKind::InvalidInput))
            .and_then(|string| {
                CString::new(string).map_err(|_| io::Error::from(io::ErrorKind::InvalidInput))
            })
            .map_err(|e| anyhow::anyhow!("invalid path: {e:?}"))?;

        let mut vfs = mem::MaybeUninit::<libc::statvfs>::uninit();

        // SAFETY: libc call, `path` is a valid C string and buf is a valid pointer to
        // write to.
        let result = unsafe { libc::statvfs(path.as_ptr(), vfs.as_mut_ptr()) };

        if result == 0 {
            // SAFETY: If result is 0, it succeeded, and vfs should be non-null.
            let vfs = unsafe { vfs.assume_init() };
            Ok(Usage::new(vfs))
        } else {
            Err(anyhow::anyhow!(
                "statvfs had an issue getting info from {path:?}"
            ))
        }
    }
}

fn fix_mount_point(s: &str) -> String {
    const ESCAPED_BACKSLASH: &str = "\\134";
    const ESCAPED_SPACE: &str = "\\040";
    const ESCAPED_TAB: &str = "\\011";
    const ESCAPED_NEWLINE: &str = "\\012";

    s.replace(ESCAPED_BACKSLASH, "\\")
        .replace(ESCAPED_SPACE, " ")
        .replace(ESCAPED_TAB, "\t")
        .replace(ESCAPED_NEWLINE, "\n")
}

impl FromStr for Partition {
    type Err = anyhow::Error;

    fn from_str(line: &str) -> anyhow::Result<Partition> {
        // Example: `/dev/sda3 /home ext4 rw,relatime,data=ordered 0 0`
        let mut parts = line.trim_start().splitn(5, ' ');

        let device = match parts.next() {
            Some("none") => None,
            Some(device) => Some(device.to_string()),
            None => {
                bail!("missing device");
            }
        };

        let mount_point = match parts.next() {
            Some(mount_point) => PathBuf::from(fix_mount_point(mount_point)),
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

        Ok(Partition {
            device,
            mount_point,
            fs_type,
        })
    }
}

#[expect(dead_code)]
/// Returns a [`Vec`] containing all partitions.
pub(crate) fn partitions() -> anyhow::Result<Vec<Partition>> {
    const PROC_MOUNTS: &str = "/proc/mounts";

    let mut results = vec![];
    let mut reader = BufReader::new(File::open(PROC_MOUNTS)?);
    let mut line = String::new();

    // This saves us from doing a string allocation on each iteration compared to
    // `lines()`.
    while let Ok(bytes) = reader.read_line(&mut line) {
        if bytes > 0 {
            if let Ok(partition) = Partition::from_str(&line) {
                results.push(partition);
            }

            line.clear();
        } else {
            break;
        }
    }

    Ok(results)
}

/// Returns a [`Vec`] containing all *physical* partitions. This is defined by
/// [`FileSystem::is_physical()`].
pub(crate) fn physical_partitions() -> anyhow::Result<Vec<Partition>> {
    const PROC_MOUNTS: &str = "/proc/mounts";

    let mut results = vec![];
    let mut reader = BufReader::new(File::open(PROC_MOUNTS)?);
    let mut line = String::new();

    // This saves us from doing a string allocation on each iteration compared to
    // `lines()`.
    while let Ok(bytes) = reader.read_line(&mut line) {
        if bytes > 0 {
            if let Ok(partition) = Partition::from_str(&line) {
                if partition.fs_type().is_physical() {
                    results.push(partition);
                }
            }

            line.clear();
        } else {
            break;
        }
    }

    Ok(results)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_fix_mount_point() {
        let line = "/run/media/test/Samsung\\040980";

        assert_eq!(fix_mount_point(line), "/run/media/test/Samsung 980");
    }
}
