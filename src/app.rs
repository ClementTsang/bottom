pub mod data_collection;
use data_collection::{processes, temperature};
use std::time::Instant;

use crate::constants;

mod process_killer;

#[derive(Clone, Copy)]
pub enum ApplicationPosition {
	Cpu,
	Mem,
	Disk,
	Temp,
	Network,
	Process,
}

pub enum ScrollDirection {
	/// UP means scrolling up --- this usually DECREMENTS
	UP,
	/// DOWN means scrolling down --- this usually INCREMENTS
	DOWN,
}

pub struct App {
	// Sorting
	pub process_sorting_type: processes::ProcessSorting,
	pub process_sorting_reverse: bool,
	pub to_be_resorted: bool,
	// Positioning
	pub currently_selected_process_position: i64,
	pub currently_selected_disk_position: i64,
	pub currently_selected_temperature_position: i64,
	pub scroll_direction: ScrollDirection,
	pub previous_disk_position: i64,
	pub previous_temp_position: i64,
	pub previous_process_position: i64,
	pub temperature_type: temperature::TemperatureType,
	pub update_rate_in_milliseconds: u64,
	pub show_average_cpu: bool,
	pub current_application_position: ApplicationPosition,
	pub data: data_collection::Data,
	awaiting_second_char: bool,
	second_char: char,
	pub use_dot: bool,
	pub show_help: bool,
	pub is_frozen: bool,
	pub left_legend: bool,
	pub use_current_cpu_total: bool,
	last_key_press: Instant,
}

impl App {
	pub fn new(
		show_average_cpu: bool, temperature_type: temperature::TemperatureType, update_rate_in_milliseconds: u64, use_dot: bool, left_legend: bool,
		use_current_cpu_total: bool,
	) -> App {
		App {
			process_sorting_type: processes::ProcessSorting::CPU,
			process_sorting_reverse: true,
			to_be_resorted: false,
			currently_selected_process_position: 0,
			currently_selected_disk_position: 0,
			currently_selected_temperature_position: 0,
			temperature_type,
			update_rate_in_milliseconds,
			show_average_cpu,
			current_application_position: ApplicationPosition::Process,
			scroll_direction: ScrollDirection::DOWN,
			previous_process_position: 0,
			previous_disk_position: 0,
			previous_temp_position: 0,
			data: data_collection::Data::default(),
			awaiting_second_char: false,
			second_char: ' ',
			use_dot,
			show_help: false,
			is_frozen: false,
			left_legend,
			use_current_cpu_total,
			last_key_press: Instant::now(),
		}
	}

	pub fn reset(&mut self) {
		self.reset_multi_tap_keys();
	}

	fn reset_multi_tap_keys(&mut self) {
		self.awaiting_second_char = false;
		self.second_char = ' ';
	}

	pub fn on_enter(&mut self) {}

	pub fn on_esc(&mut self) {
		if self.show_help {
			self.show_help = false;
		}
		self.reset_multi_tap_keys();
	}

	// TODO: How should we make it for process panel specific hotkeys?  Only if we're on process panel?  Or what?
	pub fn on_key(&mut self, caught_char: char) {
		if !self.show_help {
			let current_key_press_inst = Instant::now();
			if current_key_press_inst.duration_since(self.last_key_press).as_millis() > constants::MAX_KEY_TIMEOUT_IN_MILLISECONDS {
				self.reset_multi_tap_keys();
			}
			self.last_key_press = current_key_press_inst;

			match caught_char {
				'd' => {
					if self.awaiting_second_char && self.second_char == 'd' {
						self.awaiting_second_char = false;
						self.second_char = ' ';
						// TODO: Redo this in DD rewrite!
						self.kill_highlighted_process().unwrap_or(());
					} else {
						self.awaiting_second_char = true;
						self.second_char = 'd';
					}
				}
				'g' => {
					if self.awaiting_second_char && self.second_char == 'g' {
						self.awaiting_second_char = false;
						self.second_char = ' ';
						self.skip_to_first();
					} else {
						self.awaiting_second_char = true;
						self.second_char = 'g';
					}
				}
				'f' => {
					self.is_frozen = !self.is_frozen;
				}
				'c' => {
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

			if self.awaiting_second_char && caught_char != self.second_char {
				self.awaiting_second_char = false;
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
			ApplicationPosition::Process => ApplicationPosition::Network,
			ApplicationPosition::Disk => ApplicationPosition::Mem,
			ApplicationPosition::Temp => ApplicationPosition::Mem,
			_ => self.current_application_position,
		};
		self.reset_multi_tap_keys();
	}

	pub fn on_right(&mut self) {
		self.current_application_position = match self.current_application_position {
			ApplicationPosition::Mem => ApplicationPosition::Temp,
			ApplicationPosition::Network => ApplicationPosition::Process,
			_ => self.current_application_position,
		};
		self.reset_multi_tap_keys();
	}

	pub fn on_up(&mut self) {
		self.current_application_position = match self.current_application_position {
			ApplicationPosition::Mem => ApplicationPosition::Cpu,
			ApplicationPosition::Network => ApplicationPosition::Mem,
			ApplicationPosition::Process => ApplicationPosition::Disk,
			ApplicationPosition::Temp => ApplicationPosition::Cpu,
			ApplicationPosition::Disk => ApplicationPosition::Temp,
			_ => self.current_application_position,
		};
		self.reset_multi_tap_keys();
	}

	pub fn on_down(&mut self) {
		self.current_application_position = match self.current_application_position {
			ApplicationPosition::Cpu => ApplicationPosition::Mem,
			ApplicationPosition::Mem => ApplicationPosition::Network,
			ApplicationPosition::Temp => ApplicationPosition::Disk,
			ApplicationPosition::Disk => ApplicationPosition::Process,
			_ => self.current_application_position,
		};
		self.reset_multi_tap_keys();
	}

	pub fn skip_to_first(&mut self) {
		match self.current_application_position {
			ApplicationPosition::Process => self.currently_selected_process_position = 0,
			ApplicationPosition::Temp => self.currently_selected_temperature_position = 0,
			ApplicationPosition::Disk => self.currently_selected_disk_position = 0,
			_ => {}
		}
		self.scroll_direction = ScrollDirection::UP;
		self.reset_multi_tap_keys();
	}

	pub fn skip_to_last(&mut self) {
		match self.current_application_position {
			ApplicationPosition::Process => self.currently_selected_process_position = self.data.list_of_processes.len() as i64 - 1,
			ApplicationPosition::Temp => self.currently_selected_temperature_position = self.data.list_of_temperature_sensor.len() as i64 - 1,
			ApplicationPosition::Disk => self.currently_selected_disk_position = self.data.list_of_disks.len() as i64 - 1,
			_ => {}
		}
		self.scroll_direction = ScrollDirection::DOWN;
		self.reset_multi_tap_keys();
	}

	pub fn decrement_position_count(&mut self) {
		match self.current_application_position {
			ApplicationPosition::Process => self.change_process_position(-1),
			ApplicationPosition::Temp => self.change_temp_position(-1),
			ApplicationPosition::Disk => self.change_disk_position(-1),
			_ => {}
		}
		self.scroll_direction = ScrollDirection::UP;
		self.reset_multi_tap_keys();
	}

	pub fn increment_position_count(&mut self) {
		match self.current_application_position {
			ApplicationPosition::Process => self.change_process_position(1),
			ApplicationPosition::Temp => self.change_temp_position(1),
			ApplicationPosition::Disk => self.change_disk_position(1),
			_ => {}
		}
		self.scroll_direction = ScrollDirection::DOWN;
		self.reset_multi_tap_keys();
	}

	fn change_process_position(&mut self, num_to_change_by: i64) {
		if self.currently_selected_process_position + num_to_change_by >= 0
			&& self.currently_selected_process_position + num_to_change_by < self.data.list_of_processes.len() as i64
		{
			self.currently_selected_process_position += num_to_change_by;
		}
	}

	fn change_temp_position(&mut self, num_to_change_by: i64) {
		if self.currently_selected_temperature_position + num_to_change_by >= 0
			&& self.currently_selected_temperature_position + num_to_change_by < self.data.list_of_temperature_sensor.len() as i64
		{
			self.currently_selected_temperature_position += num_to_change_by;
		}
	}

	fn change_disk_position(&mut self, num_to_change_by: i64) {
		if self.currently_selected_disk_position + num_to_change_by >= 0
			&& self.currently_selected_disk_position + num_to_change_by < self.data.list_of_disks.len() as i64
		{
			self.currently_selected_disk_position += num_to_change_by;
		}
	}
}
