use crate::new_data_collection::sources::memory::MemData;

pub(crate) fn get_arc_usage() -> MemData {
    // TODO: [OPT] is this efficient?
    use std::fs::read_to_string;

    let (total_bytes, used_bytes) =
        if let Ok(arc_stats) = read_to_string("/proc/spl/kstat/zfs/arcstats") {
            let mut mem_arc = 0;
            let mut mem_total = 0;
            let mut zfs_keys_read: u8 = 0;
            const ZFS_KEYS_NEEDED: u8 = 2;

            for line in arc_stats.lines() {
                if let Some((label, value)) = line.split_once(' ') {
                    let to_write = match label {
                        "size" => &mut mem_arc,
                        "c_max" => &mut mem_total,
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
            (mem_total, mem_arc)
        } else {
            (0, 0)
        };

    MemData {
        used_bytes,
        total_bytes,
    }
}
