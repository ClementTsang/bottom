pub mod data_collection;
use data_collection::{processes, temperature};

pub struct App {
	pub should_quit : bool,
	pub process_sorting_type : processes::ProcessSorting,
	pub process_sorting_reverse : bool,
	pub to_be_resorted : bool,
	pub current_selected_process_position : u64,
	pub temperature_type : temperature::TemperatureType,
	pub update_rate_in_milliseconds : u64,
	pub show_average_cpu : bool,
}

impl App {
	pub fn new(show_average_cpu : bool, temperature_type : temperature::TemperatureType, update_rate_in_milliseconds : u64) -> App {
		App {
			process_sorting_type : processes::ProcessSorting::CPU,
			should_quit : false,
			process_sorting_reverse : true,
			to_be_resorted : false,
			current_selected_process_position : 0,
			temperature_type,
			update_rate_in_milliseconds,
			show_average_cpu,
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
