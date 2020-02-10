//! This is the main file to house data collection functions.

use crate::utils::error::Result;
use std::{collections::HashMap, time::Instant};
use sysinfo::{System, SystemExt};

pub mod cpu;
pub mod disks;
pub mod mem;
pub mod network;
pub mod processes;
pub mod temperature;

fn set_if_valid<T: std::clone::Clone>(result: &Result<T>, value_to_set: &mut T) {
	if let Ok(result) = result {
		*value_to_set = (*result).clone();
	}
}

#[derive(Clone, Debug)]
pub struct Data {
	pub cpu: cpu::CPUHarvest,
	pub memory: mem::MemHarvest,
	pub swap: mem::MemHarvest,
	pub temperature_sensors: Vec<temperature::TempHarvest>,
	pub network: network::NetworkHarvest,
	pub list_of_processes: Vec<processes::ProcessHarvest>,
	pub disks: Vec<disks::DiskHarvest>,
	pub io: disks::IOHarvest,
	pub last_collection_time: Instant,
}

impl Default for Data {
	fn default() -> Self {
		Data {
			cpu: cpu::CPUHarvest::default(),
			memory: mem::MemHarvest::default(),
			swap: mem::MemHarvest::default(),
			temperature_sensors: Vec::default(),
			list_of_processes: Vec::default(),
			disks: Vec::default(),
			io: disks::IOHarvest::default(),
			network: network::NetworkHarvest::default(),
			last_collection_time: Instant::now(),
		}
	}
}

impl Data {
	pub fn first_run_cleanup(&mut self) {
		self.io = disks::IOHarvest::default();
		self.temperature_sensors = Vec::new();
		self.list_of_processes = Vec::new();
		self.disks = Vec::new();

		self.network.first_run_cleanup();
		self.memory = mem::MemHarvest::default();
		self.swap = mem::MemHarvest::default();
		self.cpu = cpu::CPUHarvest::default();
	}
}

pub struct DataState {
	pub data: Data,
	sys: System,
	prev_pid_stats: HashMap<String, (f64, Instant)>,
	prev_idle: f64,
	prev_non_idle: f64,
	mem_total_kb: u64,
	temperature_type: temperature::TemperatureType,
	use_current_cpu_total: bool,
}

impl Default for DataState {
	fn default() -> Self {
		DataState {
			data: Data::default(),
			sys: System::new_all(),
			prev_pid_stats: HashMap::new(),
			prev_idle: 0_f64,
			prev_non_idle: 0_f64,
			mem_total_kb: 0,
			temperature_type: temperature::TemperatureType::Celsius,
			use_current_cpu_total: false,
		}
	}
}

impl DataState {
	pub fn set_temperature_type(&mut self, temperature_type: temperature::TemperatureType) {
		self.temperature_type = temperature_type;
	}

	pub fn set_use_current_cpu_total(&mut self, use_current_cpu_total: bool) {
		self.use_current_cpu_total = use_current_cpu_total;
	}

	pub fn init(&mut self) {
		self.mem_total_kb = self.sys.get_total_memory();
		futures::executor::block_on(self.update_data());
		std::thread::sleep(std::time::Duration::from_millis(250));
		self.data.first_run_cleanup();
	}

	pub async fn update_data(&mut self) {
		self.sys.refresh_system();

		if !cfg!(target_os = "linux") {
			// For now, might be just windows tbh
			self.sys.refresh_processes();
			self.sys.refresh_networks();
		}

		let current_instant = std::time::Instant::now();

		// Network
		self.data.network = network::get_network_data(
			&self.sys,
			&self.data.last_collection_time,
			&mut self.data.network.total_rx,
			&mut self.data.network.total_tx,
			&current_instant,
		)
		.await;

		// Mem and swap
		if let Ok(memory) = mem::get_mem_data_list().await {
			self.data.memory = memory;
		}

		if let Ok(swap) = mem::get_swap_data_list().await {
			self.data.swap = swap;
		}

		// CPU
		self.data.cpu = cpu::get_cpu_data_list(&self.sys);

		// Disks
		if let Ok(disks) = disks::get_disk_usage_list().await {
			self.data.disks = disks;
		}
		if let Ok(io) = disks::get_io_usage_list(false).await {
			self.data.io = io;
		}

		// Temp
		if let Ok(temp) = temperature::get_temperature_data(&self.sys, &self.temperature_type).await
		{
			self.data.temperature_sensors = temp;
		}

		// What we want to do: For timed data, if there is an error, just do not add.  For other data, just don't update!
		set_if_valid(
			&processes::get_sorted_processes_list(
				&self.sys,
				&mut self.prev_idle,
				&mut self.prev_non_idle,
				&mut self.prev_pid_stats,
				self.use_current_cpu_total,
				self.mem_total_kb,
				&current_instant,
			),
			&mut self.data.list_of_processes,
		);

		// Update time
		self.data.last_collection_time = current_instant;
	}
}
