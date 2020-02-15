//! This is the main file to house data collection functions.

use std::{collections::HashMap, time::Instant};
use sysinfo::{System, SystemExt};

pub mod cpu;
pub mod disks;
pub mod mem;
pub mod network;
pub mod processes;
pub mod temperature;

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
	last_collection_time: Instant,
	total_rx: u64,
	total_tx: u64,
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
			last_collection_time: Instant::now(),
			total_rx: 0,
			total_tx: 0,
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

		if cfg!(not(target_os = "linux")) {
			self.sys.refresh_processes();
			self.sys.refresh_components();
		}
		if cfg!(target_os = "windows") {
			self.sys.refresh_networks();
		}

		let current_instant = std::time::Instant::now();

		self.data.cpu = cpu::get_cpu_data_list(&self.sys);
		if let Ok(process_list) = processes::get_sorted_processes_list(
			&self.sys,
			&mut self.prev_idle,
			&mut self.prev_non_idle,
			&mut self.prev_pid_stats,
			self.use_current_cpu_total,
			self.mem_total_kb,
			current_instant,
		) {
			self.data.list_of_processes = process_list;
		}

		// ASYNC
		let network_data_fut = network::get_network_data(
			&self.sys,
			self.last_collection_time,
			&mut self.total_rx,
			&mut self.total_tx,
			current_instant,
		);

		let mem_data_fut = mem::get_mem_data_list();
		let swap_data_fut = mem::get_swap_data_list();
		let disk_data_fut = disks::get_disk_usage_list();
		let disk_io_usage_fut = disks::get_io_usage_list(false);
		let temp_data_fut = temperature::get_temperature_data(&self.sys, &self.temperature_type);

		let (net_data, mem_res, swap_res, disk_res, io_res, temp_res) = join!(
			network_data_fut,
			mem_data_fut,
			swap_data_fut,
			disk_data_fut,
			disk_io_usage_fut,
			temp_data_fut
		);

		// After async
		self.data.network = net_data;
		self.total_rx = self.data.network.total_rx;
		self.total_tx = self.data.network.total_tx;

		if let Ok(memory) = mem_res {
			self.data.memory = memory;
		}

		if let Ok(swap) = swap_res {
			self.data.swap = swap;
		}

		if let Ok(disks) = disk_res {
			self.data.disks = disks;
		}
		if let Ok(io) = io_res {
			self.data.io = io;
		}

		if let Ok(temp) = temp_res {
			self.data.temperature_sensors = temp;
		}

		// Update time
		self.data.last_collection_time = current_instant;
		self.last_collection_time = current_instant;
	}
}
