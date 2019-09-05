use sysinfo::{System, SystemExt};
mod widgets;
use widgets::{cpu, disks, mem, network, processes, temperature};

fn main() {
	let mut system = System::new();
	system.refresh_all();
	//processes::draw_sorted_processes(processes::ProcessSorting::NAME, true, &system);
	disks::draw_disk_usage_data(&system);
}
