pub mod data_collection;
use data_collection::{processes, temperature};
use std::time::Instant;

use crate::{canvas, constants, data_conversion::ConvertedProcessData, utils::error::Result};

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

#[derive(Debug)]
pub enum ScrollDirection {
	// UP means scrolling up --- this usually DECREMENTS
	UP,
	// DOWN means scrolling down --- this usually INCREMENTS
	DOWN,
}

pub struct App {
	// Sorting
	pub process_sorting_type: processes::ProcessSorting,
	pub process_sorting_reverse: bool,
	pub to_be_resorted: bool,
	// Positioning
	pub scroll_direction: ScrollDirection,
	pub currently_selected_process_position: i64,
	pub currently_selected_disk_position: i64,
	pub currently_selected_temperature_position: i64,
	pub currently_selected_cpu_table_position: i64,
	pub previous_disk_position: i64,
	pub previous_temp_position: i64,
	pub previous_process_position: i64,
	pub previous_cpu_table_position: i64,
	pub temperature_type: temperature::TemperatureType,
	pub update_rate_in_milliseconds: u64,
	pub show_average_cpu: bool,
	pub current_application_position: ApplicationPosition,
	pub data: data_collection::Data,
	awaiting_second_char: bool,
	second_char: char,
	pub use_dot: bool,
	pub show_help: bool,
	pub show_dd: bool,
	pub dd_err: Option<String>,
	to_delete_process_list: Option<Vec<ConvertedProcessData>>,
	pub is_frozen: bool,
	pub left_legend: bool,
	pub use_current_cpu_total: bool,
	last_key_press: Instant,
	pub canvas_data: canvas::CanvasData,
	enable_grouping: bool,
	enable_searching: bool,
	current_search_phrase: String,
}

impl App {
	pub fn new(
		show_average_cpu: bool, temperature_type: temperature::TemperatureType,
		update_rate_in_milliseconds: u64, use_dot: bool, left_legend: bool,
		use_current_cpu_total: bool,
	) -> App {
		App {
			process_sorting_type: processes::ProcessSorting::CPU,
			process_sorting_reverse: true,
			to_be_resorted: false,
			temperature_type,
			update_rate_in_milliseconds,
			show_average_cpu,
			current_application_position: ApplicationPosition::Process,
			scroll_direction: ScrollDirection::DOWN,
			currently_selected_process_position: 0,
			currently_selected_disk_position: 0,
			currently_selected_temperature_position: 0,
			currently_selected_cpu_table_position: 0,
			previous_process_position: 0,
			previous_disk_position: 0,
			previous_temp_position: 0,
			previous_cpu_table_position: 0,
			data: data_collection::Data::default(),
			awaiting_second_char: false,
			second_char: ' ',
			use_dot,
			show_help: false,
			show_dd: false,
			dd_err: None,
			to_delete_process_list: None,
			is_frozen: false,
			left_legend,
			use_current_cpu_total,
			last_key_press: Instant::now(),
			canvas_data: canvas::CanvasData::default(),
			enable_grouping: false,
			enable_searching: false,
			current_search_phrase: String::default(),
		}
	}

	pub fn reset(&mut self) {
		self.reset_multi_tap_keys();
		self.show_help = false;
		self.show_dd = false;
		self.enable_searching = false;
		self.to_delete_process_list = None;
		self.dd_err = None;
	}

	fn reset_multi_tap_keys(&mut self) {
		self.awaiting_second_char = false;
		self.second_char = ' ';
	}

	fn is_in_dialog(&self) -> bool {
		self.show_help || self.show_dd
	}

	pub fn toggle_grouping(&mut self) {
		// Disallow usage whilst in a dialog and only in processes
		if !self.is_in_dialog() {
			if let ApplicationPosition::Process = self.current_application_position {
				self.enable_grouping = !(self.enable_grouping);
			}
		}
	}

	pub fn on_tab(&mut self) {
		match self.current_application_position {
			ApplicationPosition::Process => self.toggle_grouping(),
			ApplicationPosition::Disk => {}
			_ => {}
		}
	}

	pub fn is_grouped(&self) -> bool {
		self.enable_grouping
	}

	pub fn toggle_searching(&mut self) {
		if !self.is_in_dialog() {
			if let ApplicationPosition::Process = self.current_application_position {
				self.enable_searching = !(self.enable_searching);
			}
		}
	}

	pub fn is_searching(&self) -> bool {
		self.enable_searching
	}

	/// One of two functions allowed to run while in a dialog...
	pub fn on_enter(&mut self) {
		if self.show_dd {
			// If within dd...
			if self.dd_err.is_none() {
				// Also ensure that we didn't just fail a dd...
				let dd_result = self.kill_highlighted_process();
				if let Err(dd_err) = dd_result {
					// There was an issue... inform the user...
					self.dd_err = Some(dd_err.to_string());
				} else {
					self.show_dd = false;
				}
			}
		}
	}

	pub fn on_char_key(&mut self, caught_char: char) {
		// Forbid any char key presses when showing a dialog box...
		if !self.is_in_dialog() {
			let current_key_press_inst = Instant::now();
			if current_key_press_inst
				.duration_since(self.last_key_press)
				.as_millis() > constants::MAX_KEY_TIMEOUT_IN_MILLISECONDS
			{
				self.reset_multi_tap_keys();
			}
			self.last_key_press = current_key_press_inst;

			match caught_char {
				'/' => {
					if let ApplicationPosition::Process = self.current_application_position {
						self.toggle_searching();
					}
				}
				'd' => {
					if let ApplicationPosition::Process = self.current_application_position {
						if self.awaiting_second_char && self.second_char == 'd' {
							self.awaiting_second_char = false;
							self.second_char = ' ';

							let current_process = if self.is_grouped() {
								let mut res: Vec<ConvertedProcessData> = Vec::new();
								for pid in &self.canvas_data.grouped_process_data
									[self.currently_selected_process_position as usize]
									.group
								{
									let result = self
										.canvas_data
										.process_data
										.iter()
										.find(|p| p.pid == *pid);

									if let Some(process) = result {
										res.push((*process).clone());
									}
								}
								res
							} else {
								vec![self.canvas_data.process_data
									[self.currently_selected_process_position as usize]
									.clone()]
							};
							self.to_delete_process_list = Some(current_process);
							self.show_dd = true;
							self.reset_multi_tap_keys();
						} else {
							self.awaiting_second_char = true;
							self.second_char = 'd';
						}
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
				'G' => self.skip_to_last(),
				'k' => self.decrement_position_count(),
				'j' => self.increment_position_count(),
				'f' => {
					self.is_frozen = !self.is_frozen;
				}
				'c' => {
					match self.process_sorting_type {
						processes::ProcessSorting::CPU => {
							self.process_sorting_reverse = !self.process_sorting_reverse
						}
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
						processes::ProcessSorting::MEM => {
							self.process_sorting_reverse = !self.process_sorting_reverse
						}
						_ => {
							self.process_sorting_type = processes::ProcessSorting::MEM;
							self.process_sorting_reverse = true;
						}
					}
					self.to_be_resorted = true;
					self.currently_selected_process_position = 0;
				}
				'p' => {
					// Disable if grouping
					if !self.enable_grouping {
						match self.process_sorting_type {
							processes::ProcessSorting::PID => {
								self.process_sorting_reverse = !self.process_sorting_reverse
							}
							_ => {
								self.process_sorting_type = processes::ProcessSorting::PID;
								self.process_sorting_reverse = false;
							}
						}
						self.to_be_resorted = true;
						self.currently_selected_process_position = 0;
					}
				}
				'n' => {
					match self.process_sorting_type {
						processes::ProcessSorting::NAME => {
							self.process_sorting_reverse = !self.process_sorting_reverse
						}
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

	pub fn kill_highlighted_process(&mut self) -> Result<()> {
		// Technically unnecessary but this is a good check...
		if let ApplicationPosition::Process = self.current_application_position {
			if let Some(current_selected_processes) = &(self.to_delete_process_list) {
				for current_selected_process in current_selected_processes {
					process_killer::kill_process_given_pid(current_selected_process.pid)?;
				}
			}
			self.to_delete_process_list = None;
		}
		Ok(())
	}

	pub fn get_current_highlighted_process_list(&self) -> Option<Vec<ConvertedProcessData>> {
		self.to_delete_process_list.clone()
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
		if !self.is_in_dialog() {
			self.current_application_position = match self.current_application_position {
				ApplicationPosition::Process => ApplicationPosition::Network,
				ApplicationPosition::Disk => ApplicationPosition::Mem,
				ApplicationPosition::Temp => ApplicationPosition::Mem,
				_ => self.current_application_position,
			};
			self.reset_multi_tap_keys();
		}
	}

	pub fn on_right(&mut self) {
		if !self.is_in_dialog() {
			self.current_application_position = match self.current_application_position {
				ApplicationPosition::Mem => ApplicationPosition::Temp,
				ApplicationPosition::Network => ApplicationPosition::Process,
				_ => self.current_application_position,
			};
			self.reset_multi_tap_keys();
		}
	}

	pub fn on_up(&mut self) {
		if !self.is_in_dialog() {
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
	}

	pub fn on_down(&mut self) {
		if !self.is_in_dialog() {
			self.current_application_position = match self.current_application_position {
				ApplicationPosition::Cpu => ApplicationPosition::Mem,
				ApplicationPosition::Mem => ApplicationPosition::Network,
				ApplicationPosition::Temp => ApplicationPosition::Disk,
				ApplicationPosition::Disk => ApplicationPosition::Process,
				_ => self.current_application_position,
			};
			self.reset_multi_tap_keys();
		}
	}

	pub fn skip_to_first(&mut self) {
		if !self.is_in_dialog() {
			match self.current_application_position {
				ApplicationPosition::Process => self.currently_selected_process_position = 0,
				ApplicationPosition::Temp => self.currently_selected_temperature_position = 0,
				ApplicationPosition::Disk => self.currently_selected_disk_position = 0,
				ApplicationPosition::Cpu => self.currently_selected_cpu_table_position = 0,

				_ => {}
			}
			self.scroll_direction = ScrollDirection::UP;
			self.reset_multi_tap_keys();
		}
	}

	pub fn skip_to_last(&mut self) {
		if !self.is_in_dialog() {
			match self.current_application_position {
				ApplicationPosition::Process => {
					self.currently_selected_process_position =
						self.data.list_of_processes.len() as i64 - 1
				}
				ApplicationPosition::Temp => {
					self.currently_selected_temperature_position =
						self.data.list_of_temperature_sensor.len() as i64 - 1
				}
				ApplicationPosition::Disk => {
					self.currently_selected_disk_position = self.data.list_of_disks.len() as i64 - 1
				}
				ApplicationPosition::Cpu => {
					if let Some(cpu_package) = self.data.list_of_cpu_packages.last() {
						if self.show_average_cpu {
							self.currently_selected_cpu_table_position =
								cpu_package.cpu_vec.len() as i64;
						} else {
							self.currently_selected_cpu_table_position =
								cpu_package.cpu_vec.len() as i64 - 1;
						}
					}
				}
				_ => {}
			}
			self.scroll_direction = ScrollDirection::DOWN;
			self.reset_multi_tap_keys();
		}
	}

	pub fn decrement_position_count(&mut self) {
		if !self.is_in_dialog() {
			match self.current_application_position {
				ApplicationPosition::Process => self.change_process_position(-1),
				ApplicationPosition::Temp => self.change_temp_position(-1),
				ApplicationPosition::Disk => self.change_disk_position(-1),
				ApplicationPosition::Cpu => self.change_cpu_table_position(-1), // TODO: Temporary, may change if we add scaling
				_ => {}
			}
			self.scroll_direction = ScrollDirection::UP;
			self.reset_multi_tap_keys();
		}
	}

	pub fn increment_position_count(&mut self) {
		if !self.is_in_dialog() {
			match self.current_application_position {
				ApplicationPosition::Process => self.change_process_position(1),
				ApplicationPosition::Temp => self.change_temp_position(1),
				ApplicationPosition::Disk => self.change_disk_position(1),
				ApplicationPosition::Cpu => self.change_cpu_table_position(1), // TODO: Temporary, may change if we add scaling
				_ => {}
			}
			self.scroll_direction = ScrollDirection::DOWN;
			self.reset_multi_tap_keys();
		}
	}

	fn change_cpu_table_position(&mut self, num_to_change_by: i64) {
		if let Some(cpu_package) = self.data.list_of_cpu_packages.last() {
			if self.currently_selected_cpu_table_position + num_to_change_by >= 0
				&& self.currently_selected_cpu_table_position + num_to_change_by
					< if self.show_average_cpu {
						cpu_package.cpu_vec.len()
					} else {
						cpu_package.cpu_vec.len() - 1
					} as i64
			{
				self.currently_selected_cpu_table_position += num_to_change_by;
			}
		}
	}

	fn change_process_position(&mut self, num_to_change_by: i64) {
		if self.currently_selected_process_position + num_to_change_by >= 0
			&& self.currently_selected_process_position + num_to_change_by
				< self.data.list_of_processes.len() as i64
		{
			self.currently_selected_process_position += num_to_change_by;
		}
	}

	fn change_temp_position(&mut self, num_to_change_by: i64) {
		if self.currently_selected_temperature_position + num_to_change_by >= 0
			&& self.currently_selected_temperature_position + num_to_change_by
				< self.data.list_of_temperature_sensor.len() as i64
		{
			self.currently_selected_temperature_position += num_to_change_by;
		}
	}

	fn change_disk_position(&mut self, num_to_change_by: i64) {
		if self.currently_selected_disk_position + num_to_change_by >= 0
			&& self.currently_selected_disk_position + num_to_change_by
				< self.data.list_of_disks.len() as i64
		{
			self.currently_selected_disk_position += num_to_change_by;
		}
	}
}
