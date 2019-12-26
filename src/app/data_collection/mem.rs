use heim::units::information;
use std::time::Instant;

#[derive(Debug, Clone)]
pub struct MemData {
	pub mem_total_in_mb: u64,
	pub mem_used_in_mb: u64,
	pub instant: Instant,
}

pub async fn get_mem_data_list() -> crate::utils::error::Result<MemData> {
	let memory = heim::memory::memory().await?;

	Ok(MemData {
		mem_total_in_mb: memory.total().get::<information::megabyte>(),
		mem_used_in_mb: memory.total().get::<information::megabyte>() - memory.available().get::<information::megabyte>(),
		instant: Instant::now(),
	})
}

pub async fn get_swap_data_list() -> crate::utils::error::Result<MemData> {
	let memory = heim::memory::swap().await?;

	Ok(MemData {
		mem_total_in_mb: memory.total().get::<information::megabyte>(),
		mem_used_in_mb: memory.used().get::<information::megabyte>(),
		instant: Instant::now(),
	})
}
