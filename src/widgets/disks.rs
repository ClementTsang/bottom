use heim_common::prelude::StreamExt;

pub struct DiskInfo {
	pub name : Box<str>,
	pub mount_point : Box<str>,
	pub avail_space : u64,
	pub total_space : u64,
}

pub struct IOInfo {
	pub name : Box<str>,
	pub read_bytes : u64,
	pub write_bytes : u64,
}

pub async fn get_io_usage_list() -> Result<Vec<IOInfo>, heim::Error> {
	let mut io_list : Vec<IOInfo> = Vec::new();
	let mut counters = heim::disk::io_counters();
	while let Some(counter) = counters.next().await {
		dbg!(counter?);
	}

	println!("\n\n--- Per physical disk ---\n");

	let mut counters = heim::disk::io_counters_physical();
	while let Some(counter) = counters.next().await {
		dbg!(counter?);
	}

	Ok(io_list)
}

pub async fn get_disk_usage_list() -> Result<Vec<DiskInfo>, heim::Error> {
	let mut vec_disks : Vec<DiskInfo> = Vec::new();
	let mut partitions_stream = heim::disk::partitions_physical();

	while let Some(part) = partitions_stream.next().await {
		let part = part?;
		let usage = heim::disk::usage(part.mount_point().to_path_buf()).await?;

		println!(
			"{:<17} {:<10} {:<10} {:<10} {:<10} {}",
			part.device().unwrap().to_str().unwrap(),
			usage.total().get::<heim_common::units::information::megabyte>(),
			usage.used().get::<heim_common::units::information::megabyte>(),
			usage.free().get::<heim_common::units::information::megabyte>(),
			part.file_system().as_str(),
			part.mount_point().to_string_lossy(),
		);
	}

	Ok(vec_disks)
}
