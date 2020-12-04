use futures::join;

#[derive(Debug, Clone)]
pub struct MemHarvest {
    pub mem_total_in_mb: u64,
    pub mem_used_in_mb: u64,
}

impl Default for MemHarvest {
    fn default() -> Self {
        MemHarvest {
            mem_total_in_mb: 0,
            mem_used_in_mb: 0,
        }
    }
}

#[cfg(any(target_arch = "aarch64", target_arch = "arm"))]
pub async fn get_mem_data(
    sys: &sysinfo::System,
) -> (
    crate::utils::error::Result<Option<MemHarvest>>,
    crate::utils::error::Result<Option<MemHarvest>>,
) {
    join!(get_ram_data(sys), get_swap_data(sys))
}

#[cfg(any(target_arch = "aarch64", target_arch = "arm"))]
pub async fn get_ram_data(
    sys: &sysinfo::System,
) -> crate::utils::error::Result<Option<MemHarvest>> {
    use sysinfo::SystemExt;

    Ok(Some(MemHarvest {
        mem_total_in_mb: sys.get_total_memory() / 1024,
        mem_used_in_mb: sys.get_used_memory() / 1024,
    }))
}

#[cfg(any(target_arch = "aarch64", target_arch = "arm"))]
pub async fn get_swap_data(
    sys: &sysinfo::System,
) -> crate::utils::error::Result<Option<MemHarvest>> {
    use sysinfo::SystemExt;

    Ok(Some(MemHarvest {
        mem_total_in_mb: sys.get_total_swap() / 1024,
        mem_used_in_mb: sys.get_used_swap() / 1024,
    }))
}

#[cfg(not(any(target_arch = "aarch64", target_arch = "arm")))]
pub async fn get_mem_data(
    actually_get: bool,
) -> (
    crate::utils::error::Result<Option<MemHarvest>>,
    crate::utils::error::Result<Option<MemHarvest>>,
) {
    if !actually_get {
        (Ok(None), Ok(None))
    } else {
        join!(get_ram_data(), get_swap_data())
    }
}

#[cfg(not(any(target_arch = "aarch64", target_arch = "arm")))]
pub async fn get_ram_data() -> crate::utils::error::Result<Option<MemHarvest>> {
    let memory = heim::memory::memory().await?;

    Ok(Some(MemHarvest {
        mem_total_in_mb: memory.total().get::<heim::units::information::megabyte>(),
        mem_used_in_mb: memory.total().get::<heim::units::information::megabyte>()
            - memory
                .available()
                .get::<heim::units::information::megabyte>(),
    }))
}

#[cfg(not(any(target_arch = "aarch64", target_arch = "arm")))]
pub async fn get_swap_data() -> crate::utils::error::Result<Option<MemHarvest>> {
    let memory = heim::memory::swap().await?;

    Ok(Some(MemHarvest {
        mem_total_in_mb: memory.total().get::<heim::units::information::megabyte>(),
        mem_used_in_mb: memory.used().get::<heim::units::information::megabyte>(),
    }))
}
