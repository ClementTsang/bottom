use heim_common::units::information;

#[derive(Clone, Default)]
pub struct MemData {
	pub mem_total : u64,
	pub mem_used : u64,
}

pub async fn get_mem_data_list() -> Result<MemData, heim::Error> {
	let memory = heim::memory::memory().await?;

	Ok(MemData {
		mem_total : memory.total().get::<information::megabyte>(),
		mem_used : memory.total().get::<information::megabyte>() - memory.available().get::<information::megabyte>(),
	})
}

pub async fn get_swap_data_list() -> Result<MemData, heim::Error> {
	let memory = heim::memory::swap().await?;

	Ok(MemData {
		mem_total : memory.total().get::<information::megabyte>(),
		mem_used : memory.used().get::<information::megabyte>(),
	})
}
