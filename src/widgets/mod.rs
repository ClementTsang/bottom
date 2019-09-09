pub mod cpu;
pub mod disks;
pub mod mem;
pub mod network;
pub mod processes;
pub mod temperature;

use sysinfo::{System, SystemExt};

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
	process_sorting_type : processes::ProcessSorting,
	process_sorting_reverse : bool,
	sys : System,
}

fn set_if_valid<T : std::clone::Clone>(result : &Result<T, heim::Error>, value_to_set : &mut T) {
	if let Ok(result) = result {
		*value_to_set = (*result).clone();
	}
}

impl<'a> App<'a> {
	pub fn new(title : &str) -> App {
		App {
			title,
			process_sorting_type : processes::ProcessSorting::NAME, // TODO: Change this based on input args...
			sys : System::new(),                                    // TODO: Evaluate whether this will cause efficiency issues...
			list_of_cpu_packages : Vec::new(),
			list_of_disks : Vec::new(),
			list_of_physical_io : Vec::new(),
			list_of_io : Vec::new(),
			list_of_processes : Vec::new(),
			list_of_temperature : Vec::new(),
			network : network::NetworkData::default(),
			memory : mem::MemData::default(),
			swap : mem::MemData::default(),
			should_quit : false,
			process_sorting_reverse : false,
		}
	}

	pub fn on_key(&mut self, c : char) {
		match c {
			'q' => self.should_quit = true,
			'c' => {
				self.process_sorting_type = processes::ProcessSorting::CPU;
				//processes::sort_processes(&self.process_sorting_type, &mut self.list_of_processes, self.process_sorting_reverse);
				// TODO: This CANNOT run while it is updating...
			}
			'm' => {
				self.process_sorting_type = processes::ProcessSorting::MEM;
				//processes::sort_processes(&self.process_sorting_type, &mut self.list_of_processes, self.process_sorting_reverse);
			}
			'p' => {
				self.process_sorting_type = processes::ProcessSorting::PID;
				//processes::sort_processes(&self.process_sorting_type, &mut self.list_of_processes, self.process_sorting_reverse);
			}
			'n' => {
				self.process_sorting_type = processes::ProcessSorting::NAME;
				//processes::sort_processes(&self.process_sorting_type, &mut self.list_of_processes, self.process_sorting_reverse);
			}
			'r' => {
				self.process_sorting_reverse = !self.process_sorting_reverse;
				//processes::sort_processes(&self.process_sorting_type, &mut self.list_of_processes, self.process_sorting_reverse);
			}
			_ => {}
		}
	}

	pub async fn update_data(&mut self) {
		self.sys.refresh_system();
		self.sys.refresh_network();

		// What we want to do: For timed data, if there is an error, just do not add.  For other data, just don't update!
		set_if_valid(&network::get_network_data(&self.sys), &mut self.network);
		set_if_valid(&cpu::get_cpu_data_list(&self.sys), &mut self.list_of_cpu_packages);

		// TODO: Joining all futures would be better...
		set_if_valid(
			&processes::get_sorted_processes_list(&self.process_sorting_type, self.process_sorting_reverse).await,
			&mut self.list_of_processes,
		);
		set_if_valid(&disks::get_disk_usage_list().await, &mut self.list_of_disks);
		set_if_valid(&disks::get_io_usage_list(false).await, &mut self.list_of_io);
		set_if_valid(&disks::get_io_usage_list(true).await, &mut self.list_of_physical_io);
		set_if_valid(&mem::get_mem_data_list().await, &mut self.memory);
		set_if_valid(&mem::get_swap_data_list().await, &mut self.swap);
		set_if_valid(&temperature::get_temperature_data().await, &mut self.list_of_temperature);
	}
}
