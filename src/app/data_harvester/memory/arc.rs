use super::MemHarvest;

/// Return ARC usage.
#[cfg(feature = "zfs")]
pub(crate) fn get_arc_usage() -> Option<MemHarvest> {
    let (mem_total_in_kib, mem_used_in_kib) = {
        cfg_if::cfg_if! {
            if #[cfg(target_os = "linux")] {
                // TODO: [OPT] is this efficient?
                use std::fs::read_to_string;
                if let Ok(arc_stats) = read_to_string("/proc/spl/kstat/zfs/arcstats") {
                    let mut mem_arc = 0;
                    let mut mem_total = 0;
                    let mut zfs_keys_read: u8 = 0;
                    const ZFS_KEYS_NEEDED: u8 = 2;

                    for line in arc_stats.lines() {
                        if let Some((label, value)) = line.split_once(' ') {
                            let to_write = match label {
                                "size" => &mut mem_arc,
                                "memory_all_bytes" => &mut mem_total,
                                _ => {
                                    continue;
                                }
                            };

                            if let Some((_type, number)) = value.trim_start().rsplit_once(' ') {
                                // Parse the value, remember it's in bytes!
                                if let Ok(number) = number.parse::<u64>() {
                                    *to_write = number;
                                    // We only need a few keys, so we can bail early.
                                    zfs_keys_read += 1;
                                    if zfs_keys_read == ZFS_KEYS_NEEDED {
                                        break;
                                    }
                                }
                            }
                        }
                    }
                    (mem_total / 1024, mem_arc / 1024)
                } else {
                    (0, 0)
                }
            } else if #[cfg(target_os = "freebsd")] {
                use sysctl::Sysctl;
                if let (Ok(mem_arc_value), Ok(mem_sys_value)) = (
                    sysctl::Ctl::new("kstat.zfs.misc.arcstats.size"),
                    sysctl::Ctl::new("hw.physmem"),
                ) {
                    if let (Ok(sysctl::CtlValue::U64(arc)), Ok(sysctl::CtlValue::Ulong(mem))) =
                        (mem_arc_value.value(), mem_sys_value.value())
                    {
                        (mem / 1024, arc / 1024)
                    } else {
                        (0, 0)
                    }
                } else {
                    (0, 0)
                }
            } else {
                (0, 0)
            }
        }
    };

    Some(MemHarvest {
        total_kib: mem_total_in_kib,
        used_kib: mem_used_in_kib,
        use_percent: if mem_total_in_kib == 0 {
            None
        } else {
            Some(mem_used_in_kib as f64 / mem_total_in_kib as f64 * 100.0)
        },
    })
}
