//! Data collection for memory via heim.

#[derive(Debug, Clone, Default)]
pub struct MemHarvest {
    pub mem_total_in_kib: u64,
    pub mem_used_in_kib: u64,
    pub use_percent: Option<f64>,
}

pub async fn get_mem_data(
    actually_get: bool,
) -> (
    crate::utils::error::Result<Option<MemHarvest>>,
    crate::utils::error::Result<Option<MemHarvest>>,
) {
    use futures::join;

    if !actually_get {
        (Ok(None), Ok(None))
    } else {
        join!(get_ram_data(), get_swap_data())
    }
}

pub async fn get_ram_data() -> crate::utils::error::Result<Option<MemHarvest>> {
    let (mem_total_in_kib, mem_used_in_kib) = {
        #[cfg(target_os = "linux")]
        {
            use smol::fs::read_to_string;
            let meminfo = read_to_string("/proc/meminfo").await?;

            // All values are in KiB by default.
            let mut mem_total = 0;
            let mut cached = 0;
            let mut s_reclaimable = 0;
            let mut shmem = 0;
            let mut buffers = 0;
            let mut mem_free = 0;

            let mut keys_read: u8 = 0;
            const TOTAL_KEYS_NEEDED: u8 = 6;

            for line in meminfo.lines() {
                if let Some((label, value)) = line.split_once(':') {
                    let to_write = match label {
                        "MemTotal" => &mut mem_total,
                        "MemFree" => &mut mem_free,
                        "Buffers" => &mut buffers,
                        "Cached" => &mut cached,
                        "Shmem" => &mut shmem,
                        "SReclaimable" => &mut s_reclaimable,
                        _ => {
                            continue;
                        }
                    };

                    if let Some((number, _unit)) = value.trim_start().split_once(' ') {
                        // Parse the value, remember it's in KiB!
                        if let Ok(number) = number.parse::<u64>() {
                            *to_write = number;

                            // We only need a few keys, so we can bail early.
                            keys_read += 1;
                            if keys_read == TOTAL_KEYS_NEEDED {
                                break;
                            }
                        }
                    }
                }
            }

            // Let's preface this by saying that memory usage calculations are... not straightforward.
            // There are conflicting implementations everywhere.
            //
            // Now that we've added this preface (mainly for future reference), the current implementation below for usage
            // is based on htop's calculation formula. See
            // https://github.com/htop-dev/htop/blob/976c6123f41492aaf613b9d172eef1842fb7b0a3/linux/LinuxProcessList.c#L1584
            // for implementation details as of writing.
            //
            // Another implementation, commonly used in other things, is to skip the shmem part of the calculation,
            // which matches gopsutil and stuff like free.

            let total = mem_total;
            let cached_mem = cached + s_reclaimable - shmem;
            let used_diff = mem_free + cached_mem + buffers;
            let used = if total >= used_diff {
                total - used_diff
            } else {
                total - mem_free
            };

            (total, used)
        }
        #[cfg(target_os = "macos")]
        {
            let memory = heim::memory::memory().await?;

            use heim::memory::os::macos::MemoryExt;
            use heim::units::information::kibibyte;
            (
                memory.total().get::<kibibyte>(),
                memory.active().get::<kibibyte>() + memory.wire().get::<kibibyte>(),
            )
        }
        #[cfg(target_os = "windows")]
        {
            let memory = heim::memory::memory().await?;

            use heim::units::information::kibibyte;
            let mem_total_in_kib = memory.total().get::<kibibyte>();
            (
                mem_total_in_kib,
                mem_total_in_kib - memory.available().get::<kibibyte>(),
            )
        }
    };

    Ok(Some(MemHarvest {
        mem_total_in_kib,
        mem_used_in_kib,
        use_percent: if mem_total_in_kib == 0 {
            None
        } else {
            Some(mem_used_in_kib as f64 / mem_total_in_kib as f64 * 100.0)
        },
    }))
}

pub async fn get_swap_data() -> crate::utils::error::Result<Option<MemHarvest>> {
    let memory = heim::memory::swap().await?;

    let (mem_total_in_kib, mem_used_in_kib) = {
        #[cfg(target_os = "linux")]
        {
            // Similar story to above - heim parses this information incorrectly as far as I can tell, so kilobytes = kibibytes here.
            use heim::units::information::kilobyte;
            (
                memory.total().get::<kilobyte>(),
                memory.used().get::<kilobyte>(),
            )
        }
        #[cfg(any(target_os = "windows", target_os = "macos"))]
        {
            use heim::units::information::kibibyte;
            (
                memory.total().get::<kibibyte>(),
                memory.used().get::<kibibyte>(),
            )
        }
    };

    Ok(Some(MemHarvest {
        mem_total_in_kib,
        mem_used_in_kib,
        use_percent: if mem_total_in_kib == 0 {
            None
        } else {
            Some(mem_used_in_kib as f64 / mem_total_in_kib as f64 * 100.0)
        },
    }))
}
