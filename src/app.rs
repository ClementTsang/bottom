pub mod data_collection;
use data_collection::{processes, temperature};

mod process_killer;

#[derive(Clone, Copy)]
pub enum ApplicationPosition {
	CPU,
	MEM,
	DISK,
	TEMP,
	NETWORK,
	PROCESS,
}

pub enum ScrollDirection {
	/// UP means scrolling up --- this usually DECREMENTS
	UP,
	/// DOWN means scrolling down --- this usually INCREMENTS
	DOWN,
}

pub struct App {
	pub process_sorting_type : processes::ProcessSorting,
	pub process_sorting_reverse : bool,
	pub to_be_resorted : bool,
	pub currently_selected_process_position : i64,
	pub currently_selected_disk_position : i64,
	pub currently_selected_temperature_position : i64,
	pub temperature_type : temperature::TemperatureType,
	pub update_rate_in_milliseconds : u64,
	pub show_average_cpu : bool,
	pub current_application_position : ApplicationPosition,
	pub current_zoom_level_percent : f64, // Make at most 200, least 50?
	pub data : data_collection::Data,
	pub scroll_direction : ScrollDirection,
	pub previous_disk_position : i64,
	pub previous_temp_position : i64,
	pub previous_process_position : i64,
	awaiting_second_d : bool,
	pub use_dot : bool,
	pub show_help : bool,
}

impl App {
	pub fn new(show_average_cpu : bool, temperature_type : temperature::TemperatureType, update_rate_in_milliseconds : u64, use_dot : bool) -> App {
		App {
			process_sorting_type : processes::ProcessSorting::CPU,
			process_sorting_reverse : true,
			to_be_resorted : false,
			currently_selected_process_position : 0,
			currently_selected_disk_position : 0,
			currently_selected_temperature_position : 0,
			temperature_type,
			update_rate_in_milliseconds,
			show_average_cpu,
			current_application_position : ApplicationPosition::PROCESS,
			current_zoom_level_percent : 100.0,
			data : data_collection::Data::default(),
			scroll_direction : ScrollDirection::DOWN,
			previous_process_position : 0,
			previous_disk_position : 0,
			previous_temp_position : 0,
			awaiting_second_d : false,
			use_dot,
			show_help : false,
		}
	}

	pub fn on_enter(&mut self) {
	}

	pub fn on_esc(&mut self) {
		if self.show_help {
			self.show_help = false;
		}
	}

	// TODO: How should we make it for process panel specific hotkeys?  Only if we're on process panel?  Or what?
	pub fn on_key(&mut self, c : char) {
		if !self.show_help {
			match c {
				'd' => {
					if self.awaiting_second_d {
						self.awaiting_second_d = false;
						self.kill_highlighted_process().unwrap_or(()); // TODO: Should this be handled?
					}
					else {
						self.awaiting_second_d = true;
					}
				}
				'c' => {
					// TODO: This should depend on what tile you're on!
					match self.process_sorting_type {
						processes::ProcessSorting::CPU => self.process_sorting_reverse = !self.process_sorting_reverse,
						_ => {
							self.process_sorting_type = processes::ProcessSorting::CPU;
							self.process_sorting_reverse = true;
						}
					}
					self.to_be_resorted = true;
					self.currently_selected_process_position = 0;
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
					self.currently_selected_process_position = 0;
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
					self.currently_selected_process_position = 0;
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
					self.currently_selected_process_position = 0;
				}
				'?' => {
					self.show_help = true;
				}
				_ => {}
			}

			if self.awaiting_second_d && c != 'd' {
				self.awaiting_second_d = false;
			}
		}
	}

	fn kill_highlighted_process(&self) -> crate::utils::error::Result<()> {
		let current_pid = u64::from(self.data.list_of_processes[self.currently_selected_process_position as usize].pid);
		process_killer::kill_process_given_pid(current_pid)?;
		Ok(())
	}

	// For now, these are hard coded --- in the future, they shouldn't be!
	//
	// General idea for now:
	// CPU -(down)> MEM
	// MEM -(down)> Network, -(right)> TEMP
	// TEMP -(down)> Disk, -(left)> MEM, -(up)> CPU
	// Disk -(down)> Processes, -(left)> MEM, -(up)> TEMP
	// Network -(up)> MEM, -(right)> PROC
	// PROC -(up)> Disk, -(left)> Network
	pub fn on_left(&mut self) {
		self.current_application_position = match self.current_application_position {
			ApplicationPosition::PROCESS => ApplicationPosition::NETWORK,
			ApplicationPosition::DISK => ApplicationPosition::MEM,
			ApplicationPosition::TEMP => ApplicationPosition::MEM,
			_ => self.current_application_position,
		};
	}

	pub fn on_right(&mut self) {
		self.current_application_position = match self.current_application_position {
			ApplicationPosition::MEM => ApplicationPosition::TEMP,
			ApplicationPosition::NETWORK => ApplicationPosition::PROCESS,
			_ => self.current_application_position,
		};
	}

	pub fn on_up(&mut self) {
		self.current_application_position = match self.current_application_position {
			ApplicationPosition::MEM => ApplicationPosition::CPU,
			ApplicationPosition::NETWORK => ApplicationPosition::MEM,
			ApplicationPosition::PROCESS => ApplicationPosition::DISK,
			ApplicationPosition::TEMP => ApplicationPosition::CPU,
			ApplicationPosition::DISK => ApplicationPosition::TEMP,
			_ => self.current_application_position,
		};
	}

	pub fn on_down(&mut self) {
		self.current_application_position = match self.current_application_position {
			ApplicationPosition::CPU => ApplicationPosition::MEM,
			ApplicationPosition::MEM => ApplicationPosition::NETWORK,
			ApplicationPosition::TEMP => ApplicationPosition::DISK,
			ApplicationPosition::DISK => ApplicationPosition::PROCESS,
			_ => self.current_application_position,
		};
	}

	pub fn decrement_position_count(&mut self) {
		match self.current_application_position {
			ApplicationPosition::PROCESS => self.change_process_position(-1),
			ApplicationPosition::TEMP => self.change_temp_position(-1),
			ApplicationPosition::DISK => self.change_disk_position(-1),
			_ => {}
		}
		self.scroll_direction = ScrollDirection::UP;
	}

	pub fn increment_position_count(&mut self) {
		match self.current_application_position {
			ApplicationPosition::PROCESS => self.change_process_position(1),
			ApplicationPosition::TEMP => self.change_temp_position(1),
			ApplicationPosition::DISK => self.change_disk_position(1),
			_ => {}
		}
		self.scroll_direction = ScrollDirection::DOWN;
	}

	fn change_process_position(&mut self, num_to_change_by : i64) {
		if self.currently_selected_process_position + num_to_change_by >= 0
			&& self.currently_selected_process_position + num_to_change_by < self.data.list_of_processes.len() as i64
		{
			self.currently_selected_process_position += num_to_change_by;
		}
	}

	fn change_temp_position(&mut self, num_to_change_by : i64) {
		if self.currently_selected_temperature_position + num_to_change_by >= 0
			&& self.currently_selected_temperature_position + num_to_change_by < self.data.list_of_temperature_sensor.len() as i64
		{
			self.currently_selected_temperature_position += num_to_change_by;
		}
	}

	fn change_disk_position(&mut self, num_to_change_by : i64) {
		if self.currently_selected_disk_position + num_to_change_by >= 0
			&& self.currently_selected_disk_position + num_to_change_by < self.data.list_of_disks.len() as i64
		{
			self.currently_selected_disk_position += num_to_change_by;
		}
	}
}
