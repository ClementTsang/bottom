//! This is the main file to house data collection functions.

use std::collections::HashMap;
use sysinfo::{System, SystemExt};

pub mod cpu;
pub mod disks;
pub mod mem;
pub mod network;
pub mod processes;
pub mod temperature;

fn set_if_valid<T : std::clone::Clone>(result : &Result<T, crate::utils::error::RustopError>, value_to_set : &mut T) {
	if let Ok(result) = result {
		*value_to_set = (*result).clone();
	}
}

fn push_if_valid<T : std::clone::Clone>(result : &Result<T, crate::utils::error::RustopError>, vector_to_push : &mut Vec<T>) {
	if let Ok(result) = result {
		vector_to_push.push(result.clone());
	}
}

#[derive(Default, Clone)]
pub struct Data {
	pub list_of_cpu_packages : Vec<cpu::CPUPackage>,
	pub list_of_io : Vec<disks::IOPackage>,
	pub list_of_physical_io : Vec<disks::IOPackage>,
	pub memory : Vec<mem::MemData>,
	pub swap : Vec<mem::MemData>,
	pub list_of_temperature_sensor : Vec<temperature::TempData>,
	pub network : Vec<network::NetworkData>,
	pub list_of_processes : Vec<processes::ProcessData>, // Only need to keep a list of processes...
	pub list_of_disks : Vec<disks::DiskData>,            // Only need to keep a list of disks and their data
}

pub struct DataState {
	pub data : Data,
	first_run : bool,
	sys : System,
	stale_max_seconds : u64,
	prev_pid_stats : HashMap<String, f64>, // TODO: Purge list?
	prev_idle : f64,
	prev_non_idle : f64,
	temperature_type : temperature::TemperatureType,
}

impl Default for DataState {
	fn default() -> Self {
		DataState {
			data : Data::default(),
			first_run : true,
			sys : System::new(),
			stale_max_seconds : 60,
			prev_pid_stats : HashMap::new(),
			prev_idle : 0_f64,
			prev_non_idle : 0_f64,
			temperature_type : temperature::TemperatureType::Celsius,
		}
	}
}

impl DataState {
	pub fn set_stale_max_seconds(&mut self, stale_max_seconds : u64) {
		self.stale_max_seconds = stale_max_seconds;
	}

	pub fn set_temperature_type(&mut self, temperature_type : temperature::TemperatureType) {
		self.temperature_type = temperature_type;
	}

	pub fn init(&mut self) {
		self.sys.refresh_system();
		self.sys.refresh_network();
	}

	pub async fn update_data(&mut self) {
		debug!("Start updating...");
		self.sys.refresh_system();
		self.sys.refresh_network();

		if !cfg!(target_os = "linux") {
			// For now, might be just windows tbh
			self.sys.refresh_processes();
		}

		// What we want to do: For timed data, if there is an error, just do not add.  For other data, just don't update!
		push_if_valid(&network::get_network_data(&self.sys), &mut self.data.network);
		push_if_valid(&cpu::get_cpu_data_list(&self.sys), &mut self.data.list_of_cpu_packages);

		// TODO: We can convert this to a multi-threaded task...
		push_if_valid(&mem::get_mem_data_list().await, &mut self.data.memory);
		push_if_valid(&mem::get_swap_data_list().await, &mut self.data.swap);
		set_if_valid(
			&processes::get_sorted_processes_list(&mut self.prev_idle, &mut self.prev_non_idle, &mut self.prev_pid_stats).await,
			&mut self.data.list_of_processes,
		);

		set_if_valid(&disks::get_disk_usage_list().await, &mut self.data.list_of_disks);
		push_if_valid(&disks::get_io_usage_list(false).await, &mut self.data.list_of_io);
		//push_if_valid(&disks::get_io_usage_list(true).await, &mut self.data.list_of_physical_io); // Removed, seems irrelevant for now...
		set_if_valid(&temperature::get_temperature_data(&self.temperature_type).await, &mut self.data.list_of_temperature_sensor);

		if self.first_run {
			self.data = Data::default();
			self.first_run = false;
		}

		// Filter out stale timed entries
		let current_instant = std::time::Instant::now();
		self.data.list_of_cpu_packages = self
			.data
			.list_of_cpu_packages
			.iter()
			.cloned()
			.filter(|entry| current_instant.duration_since(entry.instant).as_secs() <= self.stale_max_seconds)
			.collect::<Vec<_>>();

		self.data.memory = self
			.data
			.memory
			.iter()
			.cloned()
			.filter(|entry| current_instant.duration_since(entry.instant).as_secs() <= self.stale_max_seconds)
			.collect::<Vec<_>>();

		self.data.swap = self
			.data
			.swap
			.iter()
			.cloned()
			.filter(|entry| current_instant.duration_since(entry.instant).as_secs() <= self.stale_max_seconds)
			.collect::<Vec<_>>();

		self.data.network = self
			.data
			.network
			.iter()
			.cloned()
			.filter(|entry| current_instant.duration_since(entry.instant).as_secs() <= self.stale_max_seconds)
			.collect::<Vec<_>>();

		self.data.list_of_io = self
			.data
			.list_of_io
			.iter()
			.cloned()
			.filter(|entry| current_instant.duration_since(entry.instant).as_secs() <= self.stale_max_seconds)
			.collect::<Vec<_>>();

		// self.data.list_of_physical_io = self
		// 	.data
		// 	.list_of_physical_io
		// 	.iter()
		// 	.cloned()
		// 	.filter(|entry| current_instant.duration_since(entry.instant).as_secs() <= self.stale_max_seconds)
		// 	.collect::<Vec<_>>();

		debug!("End updating...");
	}
}
