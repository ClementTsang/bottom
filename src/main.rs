use sysinfo::{System, SystemExt};
mod widgets;
use widgets::{cpu, disks, mem, network, processes, temperature};

fn main() {
	// Initialize
	let mut system = System::new();
	let refresh_interval = 10;

	// Start loop (TODO: do that)
	loop {
		system.refresh_system();
		system.refresh_processes();
		system.refresh_disk_list();
		system.refresh_disks();
		system.refresh_network();

		// Get data, potentially store?
		//let list_of_processes = processes::get_sorted_processes_list(processes::ProcessSorting::NAME, true, &system);
		let list_of_disks = disks::get_disk_usage_list(&system);

		for disk in list_of_disks {
			println!("{} is mounted on {}: {}/{}", disk.name, disk.mount_point, disk.avail_space, disk.total_space);
		}

		// Draw using cursive

		// Repeat on interval
		std::thread::sleep(std::time::Duration::from_secs(refresh_interval));
	}
}
