use futures::stream::StreamExt;
use heim::units::information;
use std::time::Instant;

#[derive(Debug, Clone, Default)]
pub struct DiskData {
	pub name: Box<str>,
	pub mount_point: Box<str>,
	pub free_space: u64,
	pub used_space: u64,
	pub total_space: u64,
}

#[derive(Clone, Debug)]
pub struct IOData {
	pub mount_point: Box<str>,
	pub read_bytes: u64,
	pub write_bytes: u64,
}

#[derive(Debug, Clone)]
pub struct IOPackage {
	pub io_hash: std::collections::HashMap<String, IOData>,
	pub instant: Instant,
}

pub async fn get_io_usage_list(get_physical: bool) -> crate::utils::error::Result<IOPackage> {
	let mut io_hash: std::collections::HashMap<String, IOData> = std::collections::HashMap::new();
	if get_physical {
		let mut physical_counter_stream = heim::disk::io_counters_physical();
		while let Some(io) = physical_counter_stream.next().await {
			let io = io?;
			let mount_point = io.device_name().to_str().unwrap_or("Name Unavailable");
			io_hash.insert(
				mount_point.to_string(),
				IOData {
					mount_point: Box::from(mount_point),
					read_bytes: io.read_bytes().get::<information::megabyte>(),
					write_bytes: io.write_bytes().get::<information::megabyte>(),
				},
			);
		}
	} else {
		let mut counter_stream = heim::disk::io_counters();
		while let Some(io) = counter_stream.next().await {
			let io = io?;
			let mount_point = io.device_name().to_str().unwrap_or("Name Unavailable");
			io_hash.insert(
				mount_point.to_string(),
				IOData {
					mount_point: Box::from(mount_point),
					read_bytes: io.read_bytes().get::<information::byte>(),
					write_bytes: io.write_bytes().get::<information::byte>(),
				},
			);
		}
	}

	Ok(IOPackage {
		io_hash,
		instant: Instant::now(),
	})
}

pub async fn get_disk_usage_list() -> crate::utils::error::Result<Vec<DiskData>> {
	let mut vec_disks: Vec<DiskData> = Vec::new();
	let mut partitions_stream = heim::disk::partitions_physical();

	while let Some(part) = partitions_stream.next().await {
		if let Ok(part) = part {
			let partition = part;
			let usage = heim::disk::usage(partition.mount_point().to_path_buf()).await?;

			vec_disks.push(DiskData {
				free_space: usage.free().get::<information::byte>(),
				used_space: usage.used().get::<information::byte>(),
				total_space: usage.total().get::<information::byte>(),
				mount_point: Box::from(
					partition
						.mount_point()
						.to_str()
						.unwrap_or("Name Unavailable"),
				),
				name: Box::from(
					partition
						.device()
						.unwrap_or_else(|| std::ffi::OsStr::new("Name Unavailable"))
						.to_str()
						.unwrap_or("Name Unavailable"),
				),
			});
		}
	}

	vec_disks.sort_by(|a, b| a.name.cmp(&b.name));

	Ok(vec_disks)
}
