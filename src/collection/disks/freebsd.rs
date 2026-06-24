//! Disk stats for FreeBSD.

use std::ffi::CStr;

use rustc_hash::FxHashMap as HashMap;

use super::{DiskHarvest, IoHarvest, keep_disk_entry};
use crate::collection::{DataCollector, disks::IoData, error::CollectionResult};

/// File system types we collect usage for. This mirrors the previous
/// `df -t ufs,msdosfs,zfs` invocation that was used before we read mount
/// information directly.
const KEPT_FS_TYPES: [&str; 3] = ["ufs", "msdosfs", "zfs"];

pub fn get_io_usage(collector: &DataCollector) -> CollectionResult<IoHarvest> {
    #[cfg_attr(not(feature = "zfs"), expect(unused_mut))]
    let mut io_harvest: HashMap<String, Option<IoData>> = collector
        .sys
        .disks
        .iter()
        .map(|disk| {
            let usage = disk.usage();
            (
                disk.mount_point().to_string_lossy().to_string(),
                Some(IoData {
                    read_bytes: usage.read_bytes,
                    write_bytes: usage.written_bytes,
                }),
            )
        })
        .collect();

    #[cfg(feature = "zfs")]
    {
        use crate::collection::disks::zfs_io_counters;
        if let Ok(zfs_io) = zfs_io_counters::zfs_io_stats() {
            for io in zfs_io.into_iter() {
                let mount_point = io.device_name().to_string_lossy();
                io_harvest.insert(
                    mount_point.to_string(),
                    Some(IoData {
                        read_bytes: io.read_bytes(),
                        write_bytes: io.write_bytes(),
                    }),
                );
            }
        }
    }
    Ok(io_harvest)
}

pub fn get_disk_usage(collector: &DataCollector) -> CollectionResult<Vec<DiskHarvest>> {
    let disk_filter = &collector.filters.disk_filter;
    let mount_filter = &collector.filters.mount_filter;

    let vec_disks: Vec<DiskHarvest> = mounts()?
        .into_iter()
        .filter_map(|stat| {
            let fs_type = c_char_array_to_string(&stat.f_fstypename);
            if !KEPT_FS_TYPES.iter().any(|kept| *kept == fs_type) {
                return None;
            }

            let name = c_char_array_to_string(&stat.f_mntfromname);
            let mount_point = c_char_array_to_string(&stat.f_mntonname);

            if keep_disk_entry(&name, &mount_point, disk_filter, mount_filter) {
                // Block counts are in units of `f_bsize`, matching how `df`
                // derives the same values.
                let block_size = stat.f_bsize;
                let total_space = stat.f_blocks.saturating_mul(block_size);
                let used_space = stat
                    .f_blocks
                    .saturating_sub(stat.f_bfree)
                    .saturating_mul(block_size);
                // `f_bavail` is signed since it can go negative when a volume
                // is over its reserved threshold; clamp that to zero.
                let free_space = (stat.f_bavail.max(0) as u64).saturating_mul(block_size);

                Some(DiskHarvest {
                    free_space: Some(free_space),
                    used_space: Some(used_space),
                    total_space: Some(total_space),
                    mount_point,
                    name,
                })
            } else {
                None
            }
        })
        .collect();

    Ok(vec_disks)
}

/// Converts a null-terminated C character array (as returned by the kernel in a
/// [`libc::statfs`]) into an owned [`String`].
fn c_char_array_to_string(buf: &[libc::c_char]) -> String {
    // SAFETY: The kernel always null-terminates these fixed-size string fields.
    unsafe { CStr::from_ptr(buf.as_ptr()) }
        .to_string_lossy()
        .into_owned()
}

/// Returns all of the currently mounted filesystems via `getfsstat`, avoiding
/// the need to shell out to `df`. This is the same syscall `df` itself uses.
fn mounts() -> CollectionResult<Vec<libc::statfs>> {
    // SAFETY: System API FFI call. Passing a null buffer with size 0 just asks
    // for the number of mounted filesystems.
    let expected_len = unsafe { libc::getfsstat(std::ptr::null_mut(), 0, libc::MNT_NOWAIT) };
    if expected_len < 0 {
        return Err(std::io::Error::last_os_error().into());
    }

    let mut mounts: Vec<libc::statfs> = Vec::with_capacity(expected_len as usize);
    let bufsize = (size_of::<libc::statfs>() as libc::c_long) * (expected_len as libc::c_long);

    // SAFETY: System API FFI call. The buffer has capacity for `expected_len`
    // entries, and `bufsize` describes its size in bytes.
    let result = unsafe { libc::getfsstat(mounts.as_mut_ptr(), bufsize, libc::MNT_NOWAIT) };
    if result < 0 {
        return Err(std::io::Error::last_os_error().into());
    }

    // SAFETY: On success, `getfsstat` returns the number of `statfs` entries it
    // wrote into the buffer.
    unsafe {
        mounts.set_len(result as usize);
    }

    Ok(mounts)
}
