use std::{
    path::{Path, PathBuf},
    str::FromStr,
};

use anyhow::bail;

use crate::app::data_harvester::disks::FileSystem;

/// Representation of partition details. Based on [`heim`](https://github.com/heim-rs/heim/tree/master).
pub(crate) struct Partition {
    device: Option<String>,
    mount_point: PathBuf,
    fs_type: FileSystem,
    options: String,
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

    /// Returns the options of this partition.
    pub fn options(&self) -> &str {
        self.options.as_str()
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
        let options = match parts.next() {
            Some(opts) => opts.to_string(),
            None => {
                bail!("missing options");
            }
        };

        Ok(Partition {
            device,
            mount_point,
            fs_type,
            options,
        })
    }
}
