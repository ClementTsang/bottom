use sysinfo::{System, SystemExt};

mod widgets;
use widgets::{cpu, disks, mem, network, processes, temperature};

mod window;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
	// Initialize
	let refresh_interval = 1; // TODO: Make changing this possible!
	let mut sys = System::new();

	let mut list_of_timed_cpu_packages : Vec<cpu::TimedCPUPackages> = Vec::new();
	let mut list_of_timed_io : Vec<Vec<disks::TimedIOInfo>> = Vec::new();
	let mut list_of_timed_physical_io : Vec<Vec<disks::TimedIOInfo>> = Vec::new();
	let mut list_of_timed_memory : Vec<mem::MemData> = Vec::new();
	let mut list_of_timed_swap : Vec<mem::MemData> = Vec::new();
	let mut list_of_timed_temperature : Vec<temperature::TimedTempData> = Vec::new();

	loop {
		println!("Start data loop...");
		sys.refresh_system();

		// TODO: Get data, potentially store?  Use a result to check!
		let list_of_processes = processes::get_sorted_processes_list(processes::ProcessSorting::CPU, true).await;
		for process in list_of_processes {
			println!(
				"Process: {} with PID {}, CPU: {}%, MEM: {} MB",
				process.command, process.pid, process.cpu_usage_percent, process.mem_usage_in_mb,
			);
		}

		let list_of_disks = disks::get_disk_usage_list().await?;

		for disk in list_of_disks {
			println!("{} is mounted on {}: {}/{} free.", disk.name, disk.mount_point, disk.avail_space as f64, disk.total_space as f64);
			// TODO: Check if this is valid
		}

		list_of_timed_io.push(disks::get_io_usage_list(false).await?);
		list_of_timed_physical_io.push(disks::get_io_usage_list(true).await?);

		if !list_of_timed_io.is_empty() {
			for io in list_of_timed_io.last().unwrap() {
				println!("IO counter for {} at {:?}: {} writes, {} reads.", &io.mount_point, io.time, io.write_bytes, io.read_bytes);
			}
		}
		if !list_of_timed_physical_io.is_empty() {
			for io in list_of_timed_physical_io.last().unwrap() {
				println!("Physical IO counter for {} at {:?}: {} writes, {} reads.", &io.mount_point, io.time, io.write_bytes, io.read_bytes);
			}
		}

		list_of_timed_cpu_packages.push(cpu::get_cpu_data_list(&sys));

		if !list_of_timed_cpu_packages.is_empty() {
			let current_cpu_time = list_of_timed_cpu_packages.last().unwrap().time;
			for cpu in &list_of_timed_cpu_packages.last().unwrap().processor_list {
				println!("CPU {} has {}% usage at timestamp {:?}!", &cpu.cpu_name, cpu.cpu_usage, current_cpu_time);
			}
		}

		list_of_timed_memory.push(mem::get_mem_data_list().await?);
		list_of_timed_swap.push(mem::get_swap_data_list().await?);

		if !list_of_timed_memory.is_empty() {
			let current_mem = list_of_timed_memory.last().unwrap();
			println!("Memory usage: {} out of {} is used, at {:?}", current_mem.mem_used, current_mem.mem_total, current_mem.time);
		}

		if !list_of_timed_swap.is_empty() {
			let current_mem = list_of_timed_swap.last().unwrap();
			println!("Memory usage: {} out of {} is used, at {:?}", current_mem.mem_used, current_mem.mem_total, current_mem.time);
		}

		list_of_timed_temperature.push(temperature::get_temperature_data().await?);
		if !list_of_timed_temperature.is_empty() {
			let current_time = list_of_timed_temperature.last().unwrap().time;
			for sensor in &list_of_timed_temperature.last().unwrap().temperature_vec {
				println!("Sensor for {} is at {} degrees Celsius at timestamp {:?}!", sensor.component_name, sensor.temperature, current_time);
			}
		}

		// Send to drawing module
		println!("End data loop...");
		window::draw_terminal();

		// Repeat on interval
		std::thread::sleep(std::time::Duration::from_secs(refresh_interval));
	}

	// TODO: Exit on quit command/ctrl-c
	Ok(())
}
