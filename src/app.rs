pub mod data_collection;
use data_collection::{processes, temperature};

mod process_killer;

#[allow(dead_code)]
// Probably only use the list elements...
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
	pub should_quit : bool,
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
	pub previous_process_position : i64,
}

impl App {
	pub fn new(show_average_cpu : bool, temperature_type : temperature::TemperatureType, update_rate_in_milliseconds : u64) -> App {
		App {
			process_sorting_type : processes::ProcessSorting::CPU,
			should_quit : false,
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
		if self.currently_selected_temperature_position + num_to_change_by >= 0 {
			self.currently_selected_temperature_position += num_to_change_by;
		}
		// else if self.currently_selected_temperature_position < // TODO: Need to finish this!  This should never go PAST the number of elements
	}

	fn change_disk_position(&mut self, num_to_change_by : i64) {
		if self.currently_selected_disk_position + num_to_change_by >= 0 {
			self.currently_selected_disk_position += num_to_change_by;
		}
		// else if self.currently_selected_disk_position < // TODO: Need to finish this!  This should never go PAST the number of elements
	}
}
