use heim::units::information;

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

pub async fn get_mem_data_list(
    actually_get: bool,
) -> crate::utils::error::Result<Option<MemHarvest>> {
    if !actually_get {
        return Ok(None);
    }

    let memory = heim::memory::memory().await?;

    Ok(Some(MemHarvest {
        mem_total_in_mb: memory.total().get::<information::megabyte>(),
        mem_used_in_mb: memory.total().get::<information::megabyte>()
            - memory.available().get::<information::megabyte>(),
    }))
}

pub async fn get_swap_data_list(
    actually_get: bool,
) -> crate::utils::error::Result<Option<MemHarvest>> {
    if !actually_get {
        return Ok(None);
    }

    let memory = heim::memory::swap().await?;

    Ok(Some(MemHarvest {
        mem_total_in_mb: memory.total().get::<information::megabyte>(),
        mem_used_in_mb: memory.used().get::<information::megabyte>(),
    }))
}
