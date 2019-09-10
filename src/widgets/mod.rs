pub mod cpu;
pub mod disks;
pub mod mem;
pub mod network;
pub mod processes;
pub mod temperature;

use sysinfo::{System, SystemExt};

#[allow(dead_code)]
pub struct App<'a> {
	title : &'a str,
	pub should_quit : bool,
	pub process_sorting_type : processes::ProcessSorting,
	pub process_sorting_reverse : bool,
	pub to_be_resorted : bool,
}

fn set_if_valid<T : std::clone::Clone>(result : &Result<T, heim::Error>, value_to_set : &mut T) {
	if let Ok(result) = result {
		*value_to_set = (*result).clone();
	}
}

#[derive(Default, Clone)]
pub struct Data {
	pub list_of_cpu_packages : Vec<cpu::CPUData>,
	pub list_of_io : Vec<disks::IOData>,
	pub list_of_physical_io : Vec<disks::IOData>,
	pub memory : mem::MemData,
	pub swap : mem::MemData,
	pub list_of_temperature : Vec<temperature::TempData>,
	pub network : network::NetworkData,
	pub list_of_processes : Vec<processes::ProcessData>,
	pub list_of_disks : Vec<disks::DiskData>,
}

pub struct DataState {
	pub data : Data,
	sys : System,
}

impl Default for DataState {
	fn default() -> Self {
		DataState {
			data : Data::default(),
			sys : System::new(),
		}
	}
}

impl DataState {
	pub async fn update_data(&mut self) {
		debug!("Start updating...");
		self.sys.refresh_system();
		self.sys.refresh_network();

		// What we want to do: For timed data, if there is an error, just do not add.  For other data, just don't update!
		set_if_valid(&network::get_network_data(&self.sys), &mut self.data.network);
		set_if_valid(&cpu::get_cpu_data_list(&self.sys), &mut self.data.list_of_cpu_packages);

		// TODO: We can convert this to a multi-threaded task...
		set_if_valid(&mem::get_mem_data_list().await, &mut self.data.memory);
		set_if_valid(&mem::get_swap_data_list().await, &mut self.data.swap);
		set_if_valid(&processes::get_sorted_processes_list(self.data.memory.mem_total_in_mb).await, &mut self.data.list_of_processes);

		set_if_valid(&disks::get_disk_usage_list().await, &mut self.data.list_of_disks);
		set_if_valid(&disks::get_io_usage_list(false).await, &mut self.data.list_of_io);
		set_if_valid(&disks::get_io_usage_list(true).await, &mut self.data.list_of_physical_io);
		set_if_valid(&temperature::get_temperature_data().await, &mut self.data.list_of_temperature);
		debug!("End updating...");
	}
}

impl<'a> App<'a> {
	pub fn new(title : &str) -> App {
		App {
			title,
			process_sorting_type : processes::ProcessSorting::CPU, // TODO: Change this based on input args... basically set this on app creation
			should_quit : false,
			process_sorting_reverse : true,
			to_be_resorted : false,
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
