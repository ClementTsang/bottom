use sysinfo::{System, SystemExt};

mod widgets;
use widgets::{cpu, disks, mem, network, processes, temperature};

mod window;

fn set_if_valid<T : std::clone::Clone>(result : &Result<T, heim::Error>, value_to_set : &mut T) {
	if let Ok(result) = result {
		*value_to_set = (*result).clone();
	}
}

#[tokio::main]
async fn main() {
	// Initialize
	let refresh_interval = 1; // TODO: Make changing this possible!
	let mut sys = System::new();

	let mut list_of_cpu_packages : Vec<cpu::CPUData> = Vec::new();
	let mut list_of_io : Vec<disks::IOInfo> = Vec::new();
	let mut list_of_physical_io : Vec<disks::IOInfo> = Vec::new();
	let mut memory : mem::MemData = mem::MemData::default();
	let mut swap : mem::MemData = mem::MemData::default();
	let mut list_of_temperature : Vec<temperature::TempData> = Vec::new();
	let mut network : network::NetworkData = network::NetworkData::default();
	let mut list_of_processes = Vec::new();
	let mut list_of_disks = Vec::new();

	loop {
		println!("Start data loop...");
		sys.refresh_system();
		sys.refresh_network();

		// What we want to do: For timed data, if there is an error, just do not add.  For other data, just don't update!
		// TODO: Joining all would be better...
		set_if_valid(&network::get_network_data(&sys), &mut network);

		set_if_valid(&processes::get_sorted_processes_list(processes::ProcessSorting::NAME, false).await, &mut list_of_processes);
		set_if_valid(&disks::get_disk_usage_list().await, &mut list_of_disks);

		set_if_valid(&disks::get_io_usage_list(false).await, &mut list_of_io);
		set_if_valid(&disks::get_io_usage_list(true).await, &mut list_of_physical_io);

		set_if_valid(&mem::get_mem_data_list().await, &mut memory);
		set_if_valid(&mem::get_swap_data_list().await, &mut swap);
		set_if_valid(&temperature::get_temperature_data().await, &mut list_of_temperature);

		set_if_valid(&cpu::get_cpu_data_list(&sys), &mut list_of_cpu_packages);

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

		for io in &list_of_io {
			println!("IO counter for {}: {} writes, {} reads.", &io.mount_point, io.write_bytes, io.read_bytes);
		}

		for io in &list_of_physical_io {
			println!("Physical IO counter for {}: {} writes, {} reads.", &io.mount_point, io.write_bytes, io.read_bytes);
		}

		for cpu in &list_of_cpu_packages {
			println!("CPU {} has {}% usage!", &cpu.cpu_name, cpu.cpu_usage);
		}

		println!("Memory usage: {} out of {} is used", memory.mem_used, memory.mem_total);

		println!("Memory usage: {} out of {} is used", swap.mem_used, swap.mem_total);

		for sensor in &list_of_temperature {
			println!("Sensor for {} is at {} degrees Celsius", sensor.component_name, sensor.temperature);
		}

		println!("Network: {} rx, {} tx", network.rx, network.tx);

		// Send to drawing module
		window::draw_terminal();

		// Repeat on interval
		std::thread::sleep(std::time::Duration::from_secs(refresh_interval));
	}

	// TODO: Exit on quit command/ctrl-c
}
