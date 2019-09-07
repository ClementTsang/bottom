use sysinfo::{System, SystemExt};

mod widgets;
use widgets::{cpu, disks, mem, network, processes, temperature};

mod window;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
	// Initialize
	let refresh_interval = 1; // TODO: Make changing this possible!
	let get_physical_io = false;
	let mut sys = System::new();

	let mut list_of_timed_processes : Vec<cpu::TimedCPUPackages> = Vec::new();
	let mut list_of_timed_io : Vec<Vec<disks::TimedIOInfo>> = Vec::new();
	let mut list_of_timed_physical_io : Vec<Vec<disks::TimedIOInfo>> = Vec::new();

	loop {
		dbg!("Start data loop...");
		sys.refresh_system();

		// Get data, potentially store?
		//let list_of_processes = processes::get_sorted_processes_list(processes::ProcessSorting::NAME, true);

		let list_of_disks = disks::get_disk_usage_list().await?;

		for disk in list_of_disks {
			dbg!("{} is mounted on {}: {}/{} free.", disk.name, disk.mount_point, disk.avail_space as f64, disk.total_space as f64);
			// TODO: Check if this is valid
		}

		list_of_timed_io.push(disks::get_io_usage_list(false).await?);
		list_of_timed_physical_io.push(disks::get_io_usage_list(true).await?);

		if !list_of_timed_io.is_empty() {
			for io in list_of_timed_io.last().unwrap() {
				dbg!("IO counter for {} at {:?}: {} writes, {} reads.", &io.mount_point, io.time, io.write_bytes, io.read_bytes);
			}
		}
		if !list_of_timed_physical_io.is_empty() {
			for io in list_of_timed_physical_io.last().unwrap() {
				dbg!("Physical IO counter for {} at {:?}: {} writes, {} reads.", &io.mount_point, io.time, io.write_bytes, io.read_bytes);
			}
		}

		list_of_timed_processes.push(cpu::get_cpu_data_list(&sys));

		if !list_of_timed_processes.is_empty() {
			let current_cpu_time = list_of_timed_processes.last().unwrap().time;
			for cpu in &list_of_timed_processes.last().unwrap().processor_list {
				dbg!("CPU {} has {}% usage at timestamp {:?}!", &cpu.cpu_name, cpu.cpu_usage, current_cpu_time);
			}
		}

		// Send to drawing module
		dbg!("End data loop...");
		window::draw_terminal();

		// Repeat on interval
		std::thread::sleep(std::time::Duration::from_secs(refresh_interval));
	}

	// TODO: Exit on quit command/ctrl-c
	Ok(())
}
