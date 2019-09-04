extern crate sysinfo;
use sysinfo::{ProcessExt, SystemExt};

fn main() {
	println!("Display of processes:");

	//TODO: This clearly won't work for all systems like this
	let mut system = sysinfo::System::new();

	system.refresh_all();

	for (pid, proc) in system.get_process_list() {
		println!("{}:{} => status: {:?}", pid, proc.name(), proc.status())
	}

	println!("Current temperatures:");
	for component in system.get_components_list() {
		println!("Temp: {:?}", component);
	}

	println!(
		"Total RAM: {}, current RAM: {}, total SWAP: {}, current SWAP: {}",
		system.get_total_memory(),
		system.get_used_memory(),
		system.get_total_swap(),
		system.get_used_swap()
	);

	println!("Disk data:");
	for disk in system.get_disks() {
		println!("{:?}", disk);
	}

    println!("Network data: {:?}", system.get_network());
}
