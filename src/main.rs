use sysinfo::{System, SystemExt};

mod widgets;
use widgets::{cpu, disks, mem, network, processes, temperature};

mod window;

fn push_if_valid<T : std::clone::Clone>(result : &Result<T, heim::Error>, vector_to_push_to : &mut Vec<T>) {
	if let Ok(result) = result {
		vector_to_push_to.push(result.clone());
	}
}

#[tokio::main]
async fn main() {
	// Initialize
	let refresh_interval = 1; // TODO: Make changing this possible!
	let mut sys = System::new();

	let mut list_of_timed_cpu_packages : Vec<cpu::TimedCPUPackages> = Vec::new();
	let mut list_of_timed_io : Vec<Vec<disks::TimedIOInfo>> = Vec::new();
	let mut list_of_timed_physical_io : Vec<Vec<disks::TimedIOInfo>> = Vec::new();
	let mut list_of_timed_memory : Vec<mem::MemData> = Vec::new();
	let mut list_of_timed_swap : Vec<mem::MemData> = Vec::new();
	let mut list_of_timed_temperature : Vec<temperature::TimedTempData> = Vec::new();
	let mut list_of_timed_network : Vec<network::TimedNetworkData> = Vec::new();
	let mut list_of_processes = Vec::new();
	let mut list_of_disks = Vec::new();

	loop {
		println!("Start data loop...");
		sys.refresh_system();
		sys.refresh_network();

		// What we want to do: For timed data, if there is an error, just do not add.  For other data, just don't update!
		// TODO: Joining all would be better...
		list_of_timed_network.push(network::get_network_data(&sys));

		if let Ok(process_vec) = processes::get_sorted_processes_list(processes::ProcessSorting::CPU, true).await {
			list_of_processes = process_vec;
		}

		if let Ok(disks) = disks::get_disk_usage_list().await {
			list_of_disks = disks;
		}

		push_if_valid(&disks::get_io_usage_list(false).await, &mut list_of_timed_io);
		push_if_valid(&disks::get_io_usage_list(true).await, &mut list_of_timed_physical_io);

		push_if_valid(&mem::get_mem_data_list().await, &mut list_of_timed_memory);
		push_if_valid(&mem::get_swap_data_list().await, &mut list_of_timed_swap);
		push_if_valid(&temperature::get_temperature_data().await, &mut list_of_timed_temperature);

		push_if_valid(&cpu::get_cpu_data_list(&sys), &mut list_of_timed_cpu_packages);

		println!("End data loop...");

		// DEBUG - output results
		for process in &list_of_processes {
			println!(
				"Process: {} with PID {}, CPU: {}%, MEM: {} MB",
				process.command, process.pid, process.cpu_usage_percent, process.mem_usage_in_mb,
			);
		}
		for disk in &list_of_disks {
			println!("{} is mounted on {}: {} used.", disk.name, disk.mount_point, disk.used_space as f64 / disk.total_space as f64);
			// TODO: Check if this is valid
		}

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

		if !list_of_timed_cpu_packages.is_empty() {
			let current_cpu_time = list_of_timed_cpu_packages.last().unwrap().time;
			for cpu in &list_of_timed_cpu_packages.last().unwrap().processor_list {
				println!("CPU {} has {}% usage at timestamp {:?}!", &cpu.cpu_name, cpu.cpu_usage, current_cpu_time);
			}
		}

		if !list_of_timed_memory.is_empty() {
			let current_mem = list_of_timed_memory.last().unwrap();
			println!("Memory usage: {} out of {} is used, at {:?}", current_mem.mem_used, current_mem.mem_total, current_mem.time);
		}

		if !list_of_timed_swap.is_empty() {
			let current_mem = list_of_timed_swap.last().unwrap();
			println!("Memory usage: {} out of {} is used, at {:?}", current_mem.mem_used, current_mem.mem_total, current_mem.time);
		}

		if !list_of_timed_temperature.is_empty() {
			let current_time = list_of_timed_temperature.last().unwrap().time;
			for sensor in &list_of_timed_temperature.last().unwrap().temperature_vec {
				println!("Sensor for {} is at {} degrees Celsius at timestamp {:?}!", sensor.component_name, sensor.temperature, current_time);
			}
		}

		if !list_of_timed_network.is_empty() {
			let current_network = list_of_timed_network.last().unwrap();
			println!("Network: {} rx, {} tx at {:?}", current_network.rx, current_network.tx, current_network.time);
		}

		// Send to drawing module
		window::draw_terminal();

		// Repeat on interval
		std::thread::sleep(std::time::Duration::from_secs(refresh_interval));
	}

	// TODO: Exit on quit command/ctrl-c
}
