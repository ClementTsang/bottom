//! Data collection for memory via heim.

#[derive(Debug, Clone)]
pub struct MemHarvest {
    pub mem_total_in_kib: u64,
    pub mem_used_in_kib: u64,
}

impl Default for MemHarvest {
    fn default() -> Self {
        MemHarvest {
            mem_total_in_kib: 0,
            mem_used_in_kib: 0,
        }
    }
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
    let memory = heim::memory::memory().await?;

    let (mem_total_in_kib, mem_used_in_kib) = {
        #[cfg(target_os = "linux")]
        {
            // For Linux, the "kilobyte" value in the .total call is actually kibibytes - see
            // https://access.redhat.com/documentation/en-us/red_hat_enterprise_linux/6/html/deployment_guide/s2-proc-meminfo
            //
            // Heim parses this as kilobytes (https://github.com/heim-rs/heim/blob/master/heim-memory/src/sys/linux/memory.rs#L82)
            // even though it probably shouldn't...

            use heim::memory::os::linux::MemoryExt;
            use heim::units::information::kilobyte;
            (
                memory.total().get::<kilobyte>(),
                memory.used().get::<kilobyte>()
            )
        }
        #[cfg(target_os = "macos")]
        {
            use heim::memory::os::macos::MemoryExt;
            use heim::units::information::kibibyte;
            (
                memory.total().get::<kibibyte>(),
                memory.active().get::<kibibyte>() + memory.wire().get::<kibibyte>(),
            )
        }
        #[cfg(target_os = "windows")]
        {
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
    }))
}

pub async fn get_swap_data() -> crate::utils::error::Result<Option<MemHarvest>> {
    let memory = heim::memory::swap().await?;

    #[cfg(target_os = "linux")]
    {
        // Similar story to above - heim parses this information incorrectly, kilobytes = kibibytes here.

        use heim::units::information::kilobyte;
        Ok(Some(MemHarvest {
            mem_total_in_kib: memory.total().get::<kilobyte>(),
            mem_used_in_kib: memory.used().get::<kilobyte>(),
        }))
    }
    #[cfg(any(target_os = "windows", target_os = "macos"))]
    {
        use heim::units::information::kibibyte;
        Ok(Some(MemHarvest {
            mem_total_in_kib: memory.total().get::<kibibyte>(),
            mem_used_in_kib: memory.used().get::<kibibyte>(),
        }))
    }
}
