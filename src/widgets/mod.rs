pub mod cpu;
pub mod disks;
pub mod mem;
pub mod network;
pub mod processes;
pub mod temperature;

use sysinfo::{System, SystemExt};

#[derive(Default)]
pub struct App<'a> {
	pub should_quit : bool,
	pub list_of_cpu_packages : Vec<cpu::CPUData>,
	pub list_of_io : Vec<disks::IOData>,
	pub list_of_physical_io : Vec<disks::IOData>,
	pub memory : mem::MemData,
	pub swap : mem::MemData,
	pub list_of_temperature : Vec<temperature::TempData>,
	pub network : network::NetworkData,
	pub list_of_processes : Vec<processes::ProcessData>,
	pub list_of_disks : Vec<disks::DiskData>,
	pub title : &'a str,
}

fn set_if_valid<T : std::clone::Clone>(result : &Result<T, heim::Error>, value_to_set : &mut T) {
	if let Ok(result) = result {
		*value_to_set = (*result).clone();
	}
}

impl<'a> App<'a> {
	pub fn new(title : &str) -> App {
		let mut app = App::default();
		app.title = title;
		app
	}

	pub fn on_key(&mut self, c : char) {
		match c {
			'q' => self.should_quit = true,
			_ => {}
		}
	}

	pub async fn update_data(&mut self) {
		// Initialize
		let mut sys = System::new();

		sys.refresh_system();
		sys.refresh_network();

		// What we want to do: For timed data, if there is an error, just do not add.  For other data, just don't update!
		set_if_valid(&network::get_network_data(&sys), &mut self.network);
		set_if_valid(&cpu::get_cpu_data_list(&sys), &mut self.list_of_cpu_packages);

		// TODO: Joining all futures would be better...
		set_if_valid(&processes::get_sorted_processes_list(processes::ProcessSorting::NAME, false).await, &mut self.list_of_processes);
		set_if_valid(&disks::get_disk_usage_list().await, &mut self.list_of_disks);
		set_if_valid(&disks::get_io_usage_list(false).await, &mut self.list_of_io);
		set_if_valid(&disks::get_io_usage_list(true).await, &mut self.list_of_physical_io);
		set_if_valid(&mem::get_mem_data_list().await, &mut self.memory);
		set_if_valid(&mem::get_swap_data_list().await, &mut self.swap);
		set_if_valid(&temperature::get_temperature_data().await, &mut self.list_of_temperature);

		/*
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
		*/
	}
}
