//! Data collection about disks (e.g. I/O, usage, space).

use cfg_if::cfg_if;
use hashbrown::HashMap;

use crate::app::filter::Filter;

cfg_if! {
    if #[cfg(target_os = "freebsd")] {
        mod freebsd;
        mod io_counters;
        #[cfg(feature = "zfs")]
        mod zfs_io_counters;
        pub use io_counters::IoCounters;
        pub(crate) use self::freebsd::*;
    } else if #[cfg(target_os = "windows")] {
        mod windows;
        pub(crate) use self::windows::*;
    } else if #[cfg(target_os = "linux")] {
        mod unix;
        pub(crate) use self::unix::*;
    } else if #[cfg(target_os = "macos")] {
        mod unix;
        pub(crate) use self::unix::*;
    } else {
        mod other;
        pub(crate) use self::other::*;
    }
}

#[derive(Clone, Debug, Default)]
pub struct DiskHarvest {
    pub name: String,
    pub mount_point: String,

    /// Windows also contains an additional volume name field.
    #[cfg(target_os = "windows")]
    pub volume_name: Option<String>,

    // TODO: Maybe unify all these?
    pub free_space: Option<u64>,
    pub used_space: Option<u64>,
    pub total_space: Option<u64>,
}

#[derive(Clone, Debug)]
pub struct IoData {
    pub read_bytes: u64,
    pub write_bytes: u64,
}

pub type IoHarvest = HashMap<String, Option<IoData>>;

cfg_if! {
    if #[cfg(any(target_os = "linux", target_os = "macos", target_os = "windows"))] {
        mod io_counters;
        pub use io_counters::IoCounters;
        #[cfg(feature = "zfs")]
        mod zfs_io_counters;

        /// Returns the I/O usage of certain mount points.
        pub fn get_io_usage() -> anyhow::Result<IoHarvest> {
            let mut io_hash: HashMap<String, Option<IoData>> = HashMap::new();

            for io in io_stats()?.into_iter().flatten() {
                let mount_point = io.device_name().to_string_lossy();

                io_hash.insert(
                    mount_point.to_string(),
                    Some(IoData {
                        read_bytes: io.read_bytes(),
                        write_bytes: io.write_bytes(),
                    }),
                );
            }

            Ok(io_hash)
        }
    } else if #[cfg(not(target_os = "freebsd"))] {
        pub fn get_io_usage() -> anyhow::Result<IoHarvest> {
            anyhow::bail!("Unsupported OS");
        }
    }
}

/// Whether to keep the current disk entry given the filters, disk name, and disk mount.
/// Precedence ordering in the case where name and mount filters disagree, "allow"
/// takes precedence over "deny".
///
/// For implementation, we do this as follows:
///
/// 1. Is the entry allowed through any filter? That is, does it match an entry in a
///    filter where `is_list_ignored` is `false`? If so, we always keep this entry.
/// 2. Is the entry denied through any filter? That is, does it match an entry in a
///    filter where `is_list_ignored` is `true`? If so, we always deny this entry.
/// 3. Anything else is allowed.
pub(self) fn keep_disk_entry(
    disk_name: &str, mount_point: &str, disk_filter: &Option<Filter>, mount_filter: &Option<Filter>,
) -> bool {
    match (disk_filter, mount_filter) {
        (Some(d), Some(m)) => match (d.is_list_ignored, m.is_list_ignored) {
            (true, true) => !(d.has_match(disk_name) || m.has_match(mount_point)),
            (true, false) => {
                if m.has_match(mount_point) {
                    true
                } else {
                    d.keep_entry(disk_name)
                }
            }
            (false, true) => {
                if d.has_match(disk_name) {
                    true
                } else {
                    m.keep_entry(mount_point)
                }
            }
            (false, false) => d.has_match(disk_name) || m.has_match(mount_point),
        },
        (Some(d), None) => d.keep_entry(disk_name),
        (None, Some(m)) => m.keep_entry(mount_point),
        (None, None) => true,
    }
}

#[cfg(test)]
mod test {
    use regex::Regex;

    use super::keep_disk_entry;
    use crate::app::filter::Filter;

    fn run_filter(disk_filter: &Option<Filter>, mount_filter: &Option<Filter>) -> Vec<usize> {
        let targets = [
            ("/dev/nvme0n1p1", "/boot"),
            ("/dev/nvme0n1p2", "/"),
            ("/dev/nvme0n1p3", "/home"),
            ("/dev/sda1", "/mnt/test"),
            ("/dev/sda2", "/mnt/boot"),
        ];

        targets
            .into_iter()
            .enumerate()
            .filter_map(|(itx, (name, mount))| {
                if keep_disk_entry(name, mount, disk_filter, mount_filter) {
                    Some(itx)
                } else {
                    None
                }
            })
            .collect()
    }

    #[test]
    fn test_keeping_disk_entry() {
        let disk_ignore = Some(Filter {
            is_list_ignored: true,
            list: vec![Regex::new("nvme").unwrap()],
        });

        let disk_keep = Some(Filter {
            is_list_ignored: false,
            list: vec![Regex::new("nvme").unwrap()],
        });

        let mount_ignore = Some(Filter {
            is_list_ignored: true,
            list: vec![Regex::new("boot").unwrap()],
        });

        let mount_keep = Some(Filter {
            is_list_ignored: false,
            list: vec![Regex::new("boot").unwrap()],
        });

        assert_eq!(run_filter(&None, &None), vec![0, 1, 2, 3, 4]);

        assert_eq!(run_filter(&disk_ignore, &None), vec![3, 4]);
        assert_eq!(run_filter(&disk_keep, &None), vec![0, 1, 2]);

        assert_eq!(run_filter(&None, &mount_ignore), vec![1, 2, 3]);
        assert_eq!(run_filter(&None, &mount_keep), vec![0, 4]);

        assert_eq!(run_filter(&disk_ignore, &mount_ignore), vec![3]);
        assert_eq!(run_filter(&disk_keep, &mount_ignore), vec![0, 1, 2, 3]);

        assert_eq!(run_filter(&disk_ignore, &mount_keep), vec![0, 3, 4]);
        assert_eq!(run_filter(&disk_keep, &mount_keep), vec![0, 1, 2, 4]);
    }
}
