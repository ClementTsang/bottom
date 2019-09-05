use sysinfo::{System, SystemExt, DiskExt};

fn get_timestamped_disk_data() {}

pub fn draw_disk_usage_data(sys: &System) {
	let list_of_disks = sys.get_disks();

	for disk in list_of_disks {
		println!("Disk: Total size: {}, used: {}, disk: {}, mount: {}", disk.get_total_space(), disk.get_total_space() - disk.get_available_space(), disk.get_name().to_str().unwrap(), disk.get_mount_point().to_str().unwrap());
	}
}
