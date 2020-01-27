//! This is the main file to house data collection functions.

use crate::{constants, utils::error::Result};
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

fn push_if_valid<T: std::clone::Clone>(result: &Result<T>, vector_to_push: &mut Vec<T>) {
	if let Ok(result) = result {
		vector_to_push.push(result.clone());
	}
}

#[derive(Clone, Debug)]
pub struct Data {
	pub cpu: cpu::CPUHarvest,
	pub list_of_io: Vec<disks::IOPackage>,
	pub memory: mem::MemHarvest,
	pub swap: mem::MemHarvest,
	pub list_of_temperature_sensor: Vec<temperature::TempData>,
	pub network: network::NetworkHarvest,
	pub list_of_processes: Vec<processes::ProcessData>,
	pub grouped_list_of_processes: Option<Vec<processes::ProcessData>>,
	pub list_of_disks: Vec<disks::DiskData>,
	pub last_collection_time: Instant,
}

impl Default for Data {
	fn default() -> Self {
		Data {
			cpu: cpu::CPUHarvest::default(),
			list_of_io: Vec::default(),
			memory: mem::MemHarvest::default(),
			swap: mem::MemHarvest::default(),
			list_of_temperature_sensor: Vec::default(),
			list_of_processes: Vec::default(),
			grouped_list_of_processes: None,
			list_of_disks: Vec::default(),
			network: network::NetworkHarvest::default(),
			last_collection_time: Instant::now(),
		}
	}
}

impl Data {
	pub fn first_run_cleanup(&mut self) {
		self.list_of_io = Vec::new();
		self.list_of_temperature_sensor = Vec::new();
		self.list_of_processes = Vec::new();
		self.grouped_list_of_processes = None;
		self.list_of_disks = Vec::new();

		self.network.first_run_cleanup();
		self.memory = mem::MemHarvest::default();
		self.swap = mem::MemHarvest::default();
		self.cpu = cpu::CPUHarvest::default();
	}
}

pub struct DataState {
	pub data: Data,
	sys: System,
	stale_max_seconds: u64,
	prev_pid_stats: HashMap<String, (f64, Instant)>,
	prev_idle: f64,
	prev_non_idle: f64,
	mem_total_kb: u64,
	temperature_type: temperature::TemperatureType,
	last_clean: Instant, // Last time stale data was cleared
	use_current_cpu_total: bool,
}

impl Default for DataState {
	fn default() -> Self {
		DataState {
			data: Data::default(),
			sys: System::new(),
			stale_max_seconds: constants::STALE_MAX_MILLISECONDS / 1000,
			prev_pid_stats: HashMap::new(),
			prev_idle: 0_f64,
			prev_non_idle: 0_f64,
			mem_total_kb: 0,
			temperature_type: temperature::TemperatureType::Celsius,
			last_clean: Instant::now(),
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
		self.sys.refresh_all();
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
			self.sys.refresh_network();
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

		set_if_valid(
			&disks::get_disk_usage_list().await,
			&mut self.data.list_of_disks,
		);
		push_if_valid(
			&disks::get_io_usage_list(false).await,
			&mut self.data.list_of_io,
		);
		set_if_valid(
			&temperature::get_temperature_data(&self.sys, &self.temperature_type).await,
			&mut self.data.list_of_temperature_sensor,
		);

		self.data.last_collection_time = current_instant;

		// Filter out stale timed entries
		let clean_instant = Instant::now();
		if clean_instant.duration_since(self.last_clean).as_secs() > self.stale_max_seconds {
			let stale_list: Vec<_> = self
				.prev_pid_stats
				.iter()
				.filter(|&(_, &v)| {
					clean_instant.duration_since(v.1).as_secs() > self.stale_max_seconds
				})
				.map(|(k, _)| k.clone())
				.collect();
			for stale in stale_list {
				self.prev_pid_stats.remove(&stale);
			}
			self.data.list_of_io = self
				.data
				.list_of_io
				.iter()
				.cloned()
				.filter(|entry| {
					clean_instant.duration_since(entry.instant).as_secs() <= self.stale_max_seconds
				})
				.collect::<Vec<_>>();

			self.last_clean = clean_instant;
		}
	}
}
