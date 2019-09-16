use heim_common::prelude::StreamExt;
use std::time::Instant;

#[derive(Clone, Default)]
pub struct DiskData {
	pub name : Box<str>,
	pub mount_point : Box<str>,
	pub free_space : u64,
	pub used_space : u64,
	pub total_space : u64,
}

#[derive(Clone, Debug)]
pub struct IOData {
	pub mount_point : Box<str>,
	pub read_bytes : u64,
	pub write_bytes : u64,
}

#[derive(Clone)]
pub struct IOPackage {
	pub io_hash : std::collections::HashMap<String, IOData>,
	pub instant : Instant,
}

// TODO: This is total --- we have to change the calculation to PER SECOND!
pub async fn get_io_usage_list(get_physical : bool) -> Result<IOPackage, heim::Error> {
	let mut io_hash : std::collections::HashMap<String, IOData> = std::collections::HashMap::new();
	if get_physical {
		let mut physical_counter_stream = heim::disk::io_counters_physical();
		while let Some(io) = physical_counter_stream.next().await {
			let io = io?;
			let mount_point = io.device_name().to_str().unwrap_or("Name Unavailable");
			io_hash.insert(
				mount_point.to_string(),
				IOData {
					mount_point : Box::from(mount_point),
					read_bytes : io.read_bytes().get::<heim_common::units::information::megabyte>(),
					write_bytes : io.write_bytes().get::<heim_common::units::information::megabyte>(),
				},
			);
		}
	}
	else {
		let mut counter_stream = heim::disk::io_counters();
		while let Some(io) = counter_stream.next().await {
			let io = io?;
			let mount_point = io.device_name().to_str().unwrap_or("Name Unavailable");
			io_hash.insert(
				mount_point.to_string(),
				IOData {
					mount_point : Box::from(mount_point),
					read_bytes : io.read_bytes().get::<heim_common::units::information::byte>(),
					write_bytes : io.write_bytes().get::<heim_common::units::information::byte>(),
				},
			);
		}
	}

	Ok(IOPackage {
		io_hash,
		instant : Instant::now(),
	})
}

pub async fn get_disk_usage_list() -> Result<Vec<DiskData>, heim::Error> {
	let mut vec_disks : Vec<DiskData> = Vec::new();
	let mut partitions_stream = heim::disk::partitions_physical();

	while let Some(part) = partitions_stream.next().await {
		if let Ok(part) = part {
			let partition = part;
			let usage = heim::disk::usage(partition.mount_point().to_path_buf()).await?;

			vec_disks.push(DiskData {
				free_space : usage.free().get::<heim_common::units::information::megabyte>(),
				used_space : usage.used().get::<heim_common::units::information::megabyte>(),
				total_space : usage.total().get::<heim_common::units::information::megabyte>(),
				mount_point : Box::from(partition.mount_point().to_str().unwrap_or("Name Unavailable")),
				name : Box::from(
					partition
						.device()
						.unwrap_or_else(|| std::ffi::OsStr::new("Name Unavailable"))
						.to_str()
						.unwrap_or("Name Unavailable"),
				),
			});
		}
	}

	vec_disks.sort_by(|a, b| {
		if a.name < b.name {
			std::cmp::Ordering::Less
		}
		else if a.name > b.name {
			std::cmp::Ordering::Greater
		}
		else {
			std::cmp::Ordering::Equal
		}
	});

	Ok(vec_disks)
}
