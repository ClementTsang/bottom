#[derive(Debug, Clone)]
pub struct MemHarvest {
    pub mem_total_in_kb: u64,
    pub mem_used_in_kb: u64,
}

impl Default for MemHarvest {
    fn default() -> Self {
        MemHarvest {
            mem_total_in_kb: 0,
            mem_used_in_kb: 0,
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

    let mem_total_in_kb = memory.total().get::<heim::units::information::kilobyte>();

    Ok(Some(MemHarvest {
        mem_total_in_kb,
        mem_used_in_kb: mem_total_in_kb
            - memory
                .available()
                .get::<heim::units::information::kilobyte>(),
    }))
}

pub async fn get_swap_data() -> crate::utils::error::Result<Option<MemHarvest>> {
    let memory = heim::memory::swap().await?;

    Ok(Some(MemHarvest {
        mem_total_in_kb: memory.total().get::<heim::units::information::kilobyte>(),
        mem_used_in_kb: memory.used().get::<heim::units::information::kilobyte>(),
    }))
}
