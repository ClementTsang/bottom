use hashbrown::HashMap;

use crate::app::filter::Filter;

#[derive(Clone, Debug, Default)]
pub struct DiskEntry {
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
    /// How many bytes are read.
    pub read_bytes: u64,

    /// How many bytes are written.
    pub write_bytes: u64,
}

pub struct DiskHarvest {
    /// Disk entries.
    pub entries: Vec<DiskEntry>,
    /// I/O stats, mapped to device names.
    pub device_io_stats: HashMap<String, Option<IoData>>,
}

/// Whether to keep the current disk entry given the filters, disk name, and
/// disk mount. Precedence ordering in the case where name and mount filters
/// disagree, "allow" takes precedence over "deny".
///
/// For implementation, we do this as follows:
///
/// 1. Is the entry allowed through any filter? That is, does it match an entry
///    in a filter where `is_list_ignored` is `false`? If so, we always keep
///    this entry.
/// 2. Is the entry denied through any filter? That is, does it match an entry
///    in a filter where `is_list_ignored` is `true`? If so, we always deny this
///    entry.
/// 3. Anything else is allowed.
pub fn keep_disk_entry(
    disk_name: &str, mount_point: &str, disk_filter: &Option<Filter>, mount_filter: &Option<Filter>,
) -> bool {
    match (disk_filter, mount_filter) {
        (Some(d), Some(m)) => match (d.ignore_matches(), m.ignore_matches()) {
            (true, true) => !(d.has_match(disk_name) || m.has_match(mount_point)),
            (true, false) => {
                if m.has_match(mount_point) {
                    true
                } else {
                    d.should_keep(disk_name)
                }
            }
            (false, true) => {
                if d.has_match(disk_name) {
                    true
                } else {
                    m.should_keep(mount_point)
                }
            }
            (false, false) => d.has_match(disk_name) || m.has_match(mount_point),
        },
        (Some(d), None) => d.should_keep(disk_name),
        (None, Some(m)) => m.should_keep(mount_point),
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
        let disk_ignore = Some(Filter::new(true, vec![Regex::new("nvme").unwrap()]));
        let disk_keep = Some(Filter::new(false, vec![Regex::new("nvme").unwrap()]));
        let mount_ignore = Some(Filter::new(true, vec![Regex::new("boot").unwrap()]));
        let mount_keep = Some(Filter::new(false, vec![Regex::new("boot").unwrap()]));

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
