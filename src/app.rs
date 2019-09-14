pub mod data_collection;
use data_collection::{cpu, disks, mem, network, processes, temperature};

use std::collections::HashMap;
use sysinfo::{System, SystemExt};

#[allow(dead_code)]
pub struct App {
	pub should_quit : bool,
	pub process_sorting_type : processes::ProcessSorting,
	pub process_sorting_reverse : bool,
	pub to_be_resorted : bool,
	pub current_selected_process_position : u64,
	pub temperature_type : data_collection::temperature::TemperatureType,
	pub update_rate_in_milliseconds : u64,
}

fn set_if_valid<T : std::clone::Clone>(result : &Result<T, heim::Error>, value_to_set : &mut T) {
	if let Ok(result) = result {
		*value_to_set = (*result).clone();
	}
}

fn push_if_valid<T : std::clone::Clone>(result : &Result<T, heim::Error>, vector_to_push : &mut Vec<T>) {
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
	temperature_type : data_collection::temperature::TemperatureType,
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
			temperature_type : data_collection::temperature::TemperatureType::Celsius,
		}
	}
}

impl DataState {
	pub fn set_stale_max_seconds(&mut self, stale_max_seconds : u64) {
		self.stale_max_seconds = stale_max_seconds;
	}

	pub fn set_temperature_type(&mut self, temperature_type : data_collection::temperature::TemperatureType) {
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
		push_if_valid(&disks::get_io_usage_list(true).await, &mut self.data.list_of_physical_io);
		set_if_valid(&temperature::get_temperature_data(&self.temperature_type).await, &mut self.data.list_of_temperature_sensor);

		if self.first_run {
			self.data = Data::default();
			self.first_run = false;
		}

		// Filter out stale timed entries
		// TODO: ideally make this a generic function!
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

		self.data.list_of_physical_io = self
			.data
			.list_of_physical_io
			.iter()
			.cloned()
			.filter(|entry| current_instant.duration_since(entry.instant).as_secs() <= self.stale_max_seconds)
			.collect::<Vec<_>>();

		debug!("End updating...");
	}
}

impl App {
	pub fn new(temperature_type : data_collection::temperature::TemperatureType, update_rate_in_milliseconds : u64) -> App {
		App {
			process_sorting_type : processes::ProcessSorting::CPU, // TODO: Change this based on input args... basically set this on app creation
			should_quit : false,
			process_sorting_reverse : true,
			to_be_resorted : false,
			current_selected_process_position : 0,
			temperature_type,
			update_rate_in_milliseconds,
		}
	}

	pub fn on_key(&mut self, c : char) {
		match c {
			'q' => self.should_quit = true,
			'h' => self.on_right(),
			'j' => self.on_down(),
			'k' => self.on_up(),
			'l' => self.on_left(),
			'c' => {
				match self.process_sorting_type {
					processes::ProcessSorting::CPU => self.process_sorting_reverse = !self.process_sorting_reverse,
					_ => {
						self.process_sorting_type = processes::ProcessSorting::CPU;
						self.process_sorting_reverse = true;
					}
				}
				self.to_be_resorted = true;
			}
			'm' => {
				match self.process_sorting_type {
					processes::ProcessSorting::MEM => self.process_sorting_reverse = !self.process_sorting_reverse,
					_ => {
						self.process_sorting_type = processes::ProcessSorting::MEM;
						self.process_sorting_reverse = true;
					}
				}
				self.to_be_resorted = true;
			}
			'p' => {
				match self.process_sorting_type {
					processes::ProcessSorting::PID => self.process_sorting_reverse = !self.process_sorting_reverse,
					_ => {
						self.process_sorting_type = processes::ProcessSorting::PID;
						self.process_sorting_reverse = false;
					}
				}
				self.to_be_resorted = true;
			}
			'n' => {
				match self.process_sorting_type {
					processes::ProcessSorting::NAME => self.process_sorting_reverse = !self.process_sorting_reverse,
					_ => {
						self.process_sorting_type = processes::ProcessSorting::NAME;
						self.process_sorting_reverse = false;
					}
				}
				self.to_be_resorted = true;
			}
			_ => {}
		}
	}

	pub fn on_left(&mut self) {
	}

	pub fn on_right(&mut self) {
	}

	pub fn on_up(&mut self) {
	}

	pub fn on_down(&mut self) {
	}
}
