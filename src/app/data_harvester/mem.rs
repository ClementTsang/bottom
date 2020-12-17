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

    Ok(Some(MemHarvest {
        mem_total_in_mb: memory.total().get::<heim::units::information::megabyte>(),
        mem_used_in_mb: memory.total().get::<heim::units::information::megabyte>()
            - memory
                .available()
                .get::<heim::units::information::megabyte>(),
    }))
}

pub async fn get_swap_data() -> crate::utils::error::Result<Option<MemHarvest>> {
    let memory = heim::memory::swap().await?;

    Ok(Some(MemHarvest {
        mem_total_in_mb: memory.total().get::<heim::units::information::megabyte>(),
        mem_used_in_mb: memory.used().get::<heim::units::information::megabyte>(),
    }))
}
