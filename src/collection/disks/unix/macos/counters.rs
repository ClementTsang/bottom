//! Based on [heim's implementation](https://github.com/heim-rs/heim/blob/master/heim-disk/src/sys/macos/counters.rs).

use super::io_kit::{self, get_dict, get_disks, get_i64, get_string};
use crate::collection::disks::IoCounters;

fn get_device_io(device: io_kit::IoObject) -> anyhow::Result<IoCounters> {
    let parent = device.service_parent()?;

    // XXX: Re: Conform check being disabled.
    //
    // Okay, so this is weird.
    //
    // The problem is that if I have this check - this is what sources like psutil
    // use, for example (see https://github.com/giampaolo/psutil/blob/7eadee31db2f038763a3a6f978db1ea76bbc4674/psutil/_psutil_osx.c#LL1422C20-L1422C20)
    // then this will only return stuff like disk0.
    //
    // The problem with this is that there is *never* a disk0 *disk* entry to
    // correspond to this, so there will be entries like disk1 or whatnot.
    // Someone's done some digging on the gopsutil repo (https://github.com/shirou/gopsutil/issues/855#issuecomment-610016435), and it seems
    // like this is a consequence of how Apple does logical volumes.
    //
    // So with all that said, what I've found is that I *can* still get a mapping -
    // but I have to disable the conform check, which... is weird. I'm not sure
    // if this is valid at all. But it *does* seem to match Activity Monitor
    // with regards to disk activity, so... I guess we can leave this for
    // now...?

    // if !parent.conforms_to_block_storage_driver() {
    //     anyhow::bail!("{parent:?}, the parent of {device:?} does not conform to
    // IOBlockStorageDriver") }

    let disk_props = device.properties()?;
    let parent_props = parent.properties()?;

    let name = get_string(&disk_props, "BSD Name")?;
    let stats = get_dict(&parent_props, "Statistics")?;

    let read_bytes = get_i64(&stats, "Bytes (Read)")? as u64;
    let write_bytes = get_i64(&stats, "Bytes (Write)")? as u64;

    // let read_count = stats.get_i64("Operations (Read)")? as u64;
    // let write_count = stats.get_i64("Operations (Write)")? as u64;

    Ok(IoCounters::new(name, read_bytes, write_bytes))
}

/// Returns an iterator of disk I/O stats. Pulls data through IOKit.
pub fn io_stats() -> anyhow::Result<Vec<IoCounters>> {
    Ok(get_disks()?.filter_map(|d| get_device_io(d).ok()).collect())
}
