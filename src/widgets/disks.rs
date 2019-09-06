use sysinfo::{System, SystemExt, Disk, DiskExt};

pub struct DiskInfo<'a> {
	pub name: &'a str,
	pub mount_point: &'a str,
	pub avail_space: u64,
	pub total_space: u64,
}

pub fn get_disk_usage_list(sys: &System) -> Vec<DiskInfo> {
	let result_disks = sys.get_disks();
	let mut vec_disks : Vec<DiskInfo> = Vec::new();

	for disk in result_disks {
		vec_disks.push(DiskInfo {
			name: disk.get_name().to_str().unwrap(),
			mount_point: disk.get_mount_point().to_str().unwrap(),
			avail_space: disk.get_available_space(),
			total_space: disk.get_total_space(),
		});
	}

	vec_disks
}
