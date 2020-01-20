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

#[derive(Clone, Debug, Default)]
pub struct Data {
	pub list_of_cpu_packages: Vec<cpu::CPUPackage>,
	pub list_of_io: Vec<disks::IOPackage>,
	pub list_of_physical_io: Vec<disks::IOPackage>,
	pub memory: Vec<mem::MemData>,
	pub swap: Vec<mem::MemData>,
	pub list_of_temperature_sensor: Vec<temperature::TempData>,
	pub network: network::NetworkStorage,
	pub list_of_processes: Vec<processes::ProcessData>,
	pub grouped_list_of_processes: Option<Vec<processes::ProcessData>>,
	pub list_of_disks: Vec<disks::DiskData>,
}

pub struct DataState {
	pub data: Data,
	first_run: bool,
	sys: System,
	stale_max_seconds: u64,
	prev_pid_stats: HashMap<String, (f64, Instant)>,
	prev_idle: f64,
	prev_non_idle: f64,
	temperature_type: temperature::TemperatureType,
	last_clean: Instant, // Last time stale data was cleared
	use_current_cpu_total: bool,
}

impl Default for DataState {
	fn default() -> Self {
		DataState {
			data: Data::default(),
			first_run: true,
			sys: System::new(),
			stale_max_seconds: constants::STALE_MAX_MILLISECONDS / 1000,
			prev_pid_stats: HashMap::new(),
			prev_idle: 0_f64,
			prev_non_idle: 0_f64,
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
		let new_network_data = network::get_network_data(
			&self.sys,
			&self.data.network.last_collection_time,
			&mut self.data.network.total_rx,
			&mut self.data.network.total_tx,
			&current_instant,
		)
		.await;

		let joining_points: Option<Vec<network::NetworkJoinPoint>> =
			if !self.data.network.data_points.is_empty() {
				if let Some(prev_data) = self
					.data
					.network
					.data_points
					.get(&self.data.network.last_collection_time)
				{
					// If not empty, inject joining points

					let rx_diff = new_network_data.rx as f64 - prev_data.0.rx as f64;
					let tx_diff = new_network_data.tx as f64 - prev_data.0.tx as f64;
					let time_gap = current_instant
						.duration_since(self.data.network.last_collection_time)
						.as_millis() as f64;

					let mut new_joining_points = Vec::new();

					for idx in 0..100 {
						new_joining_points.push(network::NetworkJoinPoint {
							rx: prev_data.0.rx as f64 + rx_diff / 100.0 * idx as f64,
							tx: prev_data.0.tx as f64 + tx_diff / 100.0 * idx as f64,
							time_offset_milliseconds: time_gap / 100.0 * (100 - idx) as f64,
						});
					}
					Some(new_joining_points)
				} else {
					None
				}
			} else {
				None
			};

		// Set values
		self.data.network.rx = new_network_data.rx;
		self.data.network.tx = new_network_data.tx;
		self.data.network.last_collection_time = current_instant;

		// Add new point
		self.data
			.network
			.data_points
			.insert(current_instant, (new_network_data, joining_points));

		// What we want to do: For timed data, if there is an error, just do not add.  For other data, just don't update!
		push_if_valid(
			&cpu::get_cpu_data_list(&self.sys, &current_instant),
			&mut self.data.list_of_cpu_packages,
		);

		push_if_valid(
			&mem::get_mem_data_list(&current_instant).await,
			&mut self.data.memory,
		);
		push_if_valid(
			&mem::get_swap_data_list(&current_instant).await,
			&mut self.data.swap,
		);
		set_if_valid(
			&processes::get_sorted_processes_list(
				&self.sys,
				&mut self.prev_idle,
				&mut self.prev_non_idle,
				&mut self.prev_pid_stats,
				self.use_current_cpu_total,
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

		if self.first_run {
			self.data = Data::default();
			self.first_run = false;
		}

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

			// TODO: [OPT] cleaning stale network

			self.data.list_of_cpu_packages = self
				.data
				.list_of_cpu_packages
				.iter()
				.cloned()
				.filter(|entry| {
					clean_instant.duration_since(entry.instant).as_secs() <= self.stale_max_seconds
				})
				.collect::<Vec<_>>();

			self.data.memory = self
				.data
				.memory
				.iter()
				.cloned()
				.filter(|entry| {
					clean_instant.duration_since(entry.instant).as_secs() <= self.stale_max_seconds
				})
				.collect::<Vec<_>>();

			self.data.swap = self
				.data
				.swap
				.iter()
				.cloned()
				.filter(|entry| {
					clean_instant.duration_since(entry.instant).as_secs() <= self.stale_max_seconds
				})
				.collect::<Vec<_>>();

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
