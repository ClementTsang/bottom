use heim_common::prelude::StreamExt;

pub struct DiskInfo {
	pub name : Box<str>,
	pub mount_point : Box<str>,
	pub avail_space : u64,
	pub total_space : u64,
}

pub struct TimedIOInfo {
	pub mount_point : Box<str>,
	pub read_bytes : u64,
	pub write_bytes : u64,
	pub time : std::time::SystemTime,
}

pub async fn get_io_usage_list(get_physical : bool) -> Result<Vec<TimedIOInfo>, heim::Error> {
	let mut io_list : Vec<TimedIOInfo> = Vec::new();
	if get_physical {
		let mut physical_counter_stream = heim::disk::io_counters_physical();
		while let Some(io) = physical_counter_stream.next().await {
			let io = io?;
			io_list.push(TimedIOInfo {
				mount_point : Box::from(io.device_name().to_str().unwrap_or("Name Unavailable")),
				read_bytes : io.read_bytes().get::<heim_common::units::information::megabyte>(),
				write_bytes : io.write_bytes().get::<heim_common::units::information::megabyte>(),
				time : std::time::SystemTime::now(),
			})
		}
	}
	else {
		let mut counter_stream = heim::disk::io_counters();
		while let Some(io) = counter_stream.next().await {
			let io = io?;
			io_list.push(TimedIOInfo {
				mount_point : Box::from(io.device_name().to_str().unwrap_or("Name Unavailable")),
				read_bytes : io.read_bytes().get::<heim_common::units::information::megabyte>(),
				write_bytes : io.write_bytes().get::<heim_common::units::information::megabyte>(),
				time : std::time::SystemTime::now(),
			})
		}
	}

	Ok(io_list)
}

pub fn is_io_data_old() -> bool {
	true
}

pub async fn get_disk_usage_list() -> Result<Vec<DiskInfo>, heim::Error> {
	let mut vec_disks : Vec<DiskInfo> = Vec::new();
	let mut partitions_stream = heim::disk::partitions_physical();

	while let Some(part) = partitions_stream.next().await {
		let partition = part?; // TODO: Change this?  We don't want to error out immediately...
		let usage = heim::disk::usage(partition.mount_point().to_path_buf()).await?;

		vec_disks.push(DiskInfo {
			avail_space : usage.free().get::<heim_common::units::information::megabyte>(),
			total_space : usage.total().get::<heim_common::units::information::megabyte>(),
			mount_point : Box::from(partition.mount_point().to_str().unwrap_or("Name Unavailable")),
			name : Box::from(partition.device().unwrap_or_else(|| std::ffi::OsStr::new("Name Unavailable")).to_str().unwrap_or("Name Unavailable")),
		});
	}

	Ok(vec_disks)
}
