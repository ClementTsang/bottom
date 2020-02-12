pub mod data_harvester;
use data_harvester::{processes, temperature};
use std::time::Instant;

pub mod data_farmer;
use data_farmer::*;

use crate::{canvas, constants, utils::error::Result};

mod process_killer;

#[derive(Debug, Clone, Copy)]
pub enum WidgetPosition {
	Cpu,
	Mem,
	Disk,
	Temp,
	Network,
	Process,
	ProcessSearch,
}

#[derive(Debug)]
pub enum ScrollDirection {
	// UP means scrolling up --- this usually DECREMENTS
	UP,
	// DOWN means scrolling down --- this usually INCREMENTS
	DOWN,
}

lazy_static! {
	static ref BASE_REGEX: std::result::Result<regex::Regex, regex::Error> =
		regex::Regex::new(".*");
}

/// AppScrollWidgetState deals with fields for a scrollable app's current state.
#[derive(Default)]
pub struct AppScrollWidgetState {
	pub current_scroll_position: u64,
	pub previous_scroll_position: u64,
}

pub struct AppScrollState {
	pub scroll_direction: ScrollDirection,
	pub process_scroll_state: AppScrollWidgetState,
	pub disk_scroll_state: AppScrollWidgetState,
	pub temp_scroll_state: AppScrollWidgetState,
	pub cpu_scroll_state: AppScrollWidgetState,
}

impl Default for AppScrollState {
	fn default() -> Self {
		AppScrollState {
			scroll_direction: ScrollDirection::DOWN,
			process_scroll_state: AppScrollWidgetState::default(),
			disk_scroll_state: AppScrollWidgetState::default(),
			temp_scroll_state: AppScrollWidgetState::default(),
			cpu_scroll_state: AppScrollWidgetState::default(),
		}
	}
}

/// AppSearchState only deals with the search's current settings and state.
pub struct AppSearchState {
	current_search_query: String,
	searching_pid: bool,
	ignore_case: bool,
	current_regex: std::result::Result<regex::Regex, regex::Error>,
	current_cursor_position: usize,
	match_word: bool,
	use_regex: bool,
}

impl Default for AppSearchState {
	fn default() -> Self {
		AppSearchState {
			current_search_query: String::default(),
			searching_pid: false,
			ignore_case: true,
			current_regex: BASE_REGEX.clone(),
			current_cursor_position: 0,
			match_word: false,
			use_regex: false,
		}
	}
}

impl AppSearchState {
	pub fn toggle_ignore_case(&mut self) {
		self.ignore_case = !self.ignore_case;
	}

	pub fn toggle_search_whole_word(&mut self) {
		self.match_word = !self.match_word;
	}

	pub fn toggle_search_regex(&mut self) {
		self.use_regex = !self.use_regex;
	}

	pub fn toggle_search_with_pid(&mut self) {
		self.searching_pid = !self.searching_pid;
	}

	pub fn is_ignoring_case(&self) -> bool {
		self.ignore_case
	}

	pub fn is_searching_whole_word(&self) -> bool {
		self.match_word
	}

	pub fn is_searching_with_regex(&self) -> bool {
		self.use_regex
	}

	pub fn is_searching_with_pid(&self) -> bool {
		self.searching_pid
	}
}

#[derive(Default)]
pub struct AppDeleteDialogState {
	pub is_showing_dd: bool,
	pub is_on_yes: bool, // Defaults to "No"
}

pub enum AppHelpCategory {
	General,
	Process,
	Search,
}

pub struct AppHelpDialogState {
	pub is_showing_help: bool,
	pub current_category: AppHelpCategory,
}

impl Default for AppHelpDialogState {
	fn default() -> Self {
		AppHelpDialogState {
			is_showing_help: false,
			current_category: AppHelpCategory::General,
		}
	}
}

/// AppConfigFields is meant to cover basic fields that would normally be set
/// by config files or launch options.  Don't need to be mutable (set and forget).
pub struct AppConfigFields {
	pub update_rate_in_milliseconds: u64,
	pub temperature_type: temperature::TemperatureType,
	pub use_dot: bool,
	pub left_legend: bool,
	pub show_average_cpu: bool,
	pub use_current_cpu_total: bool,
}

pub struct App {
	pub process_sorting_type: processes::ProcessSorting,
	pub process_sorting_reverse: bool,
	pub update_process_gui: bool,
	pub app_scroll_positions: AppScrollState,
	pub current_widget_selected: WidgetPosition,
	pub data: data_harvester::Data,
	awaiting_second_char: bool,
	second_char: Option<char>,
	pub dd_err: Option<String>,
	to_delete_process_list: Option<(String, Vec<u32>)>,
	pub is_frozen: bool,
	last_key_press: Instant,
	pub canvas_data: canvas::DisplayableData,
	enable_grouping: bool,
	enable_searching: bool,
	pub data_collection: DataCollection,
	pub search_state: AppSearchState,
	pub delete_dialog_state: AppDeleteDialogState,
	pub help_dialog_state: AppHelpDialogState,
	pub app_config_fields: AppConfigFields,
	pub is_expanded: bool,
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
			update_process_gui: false,
			current_widget_selected: WidgetPosition::Process,
			app_scroll_positions: AppScrollState::default(),
			data: data_harvester::Data::default(),
			awaiting_second_char: false,
			second_char: None,
			dd_err: None,
			to_delete_process_list: None,
			is_frozen: false,
			last_key_press: Instant::now(),
			canvas_data: canvas::DisplayableData::default(),
			enable_grouping: false,
			enable_searching: false,
			data_collection: DataCollection::default(),
			search_state: AppSearchState::default(),
			delete_dialog_state: AppDeleteDialogState::default(),
			help_dialog_state: AppHelpDialogState::default(),
			app_config_fields: AppConfigFields {
				show_average_cpu,
				temperature_type,
				use_dot,
				update_rate_in_milliseconds,
				left_legend,
				use_current_cpu_total,
			},
			is_expanded: false,
		}
	}

	pub fn reset(&mut self) {
		self.reset_multi_tap_keys();
		self.help_dialog_state.is_showing_help = false;
		self.delete_dialog_state.is_showing_dd = false;
		if self.enable_searching {
			self.current_widget_selected = WidgetPosition::Process;
			self.enable_searching = false;
		}
		self.search_state.current_search_query = String::new();
		self.search_state.searching_pid = false;
		self.to_delete_process_list = None;
		self.dd_err = None;
	}

	pub fn on_esc(&mut self) {
		self.reset_multi_tap_keys();
		if self.is_in_dialog() {
			self.help_dialog_state.is_showing_help = false;
			self.help_dialog_state.current_category = AppHelpCategory::General;
			self.delete_dialog_state.is_showing_dd = false;
			self.delete_dialog_state.is_on_yes = false;
			self.to_delete_process_list = None;
			self.dd_err = None;
		} else if self.enable_searching {
			self.current_widget_selected = WidgetPosition::Process;
			self.enable_searching = false;
		} else if self.is_expanded {
			self.is_expanded = false;
		}
	}

	fn reset_multi_tap_keys(&mut self) {
		self.awaiting_second_char = false;
		self.second_char = None;
	}

	fn is_in_dialog(&self) -> bool {
		self.help_dialog_state.is_showing_help || self.delete_dialog_state.is_showing_dd
	}

	pub fn toggle_grouping(&mut self) {
		// Disallow usage whilst in a dialog and only in processes
		if !self.is_in_dialog() {
			if let WidgetPosition::Process = self.current_widget_selected {
				self.enable_grouping = !(self.enable_grouping);
				self.update_process_gui = true;
			}
		}
	}

	pub fn on_tab(&mut self) {
		match self.current_widget_selected {
			WidgetPosition::Process => self.toggle_grouping(),
			WidgetPosition::Disk => {}
			WidgetPosition::ProcessSearch => {
				if self.search_state.is_searching_with_pid() {
					self.search_with_name();
				} else {
					self.search_with_pid();
				}
			}
			_ => {}
		}
	}

	pub fn is_grouped(&self) -> bool {
		self.enable_grouping
	}

	pub fn enable_searching(&mut self) {
		if !self.is_in_dialog() {
			match self.current_widget_selected {
				WidgetPosition::Process | WidgetPosition::ProcessSearch => {
					// Toggle on
					self.enable_searching = true;
					self.current_widget_selected = WidgetPosition::ProcessSearch;
				}
				_ => {}
			}
		}
	}

	pub fn is_searching(&self) -> bool {
		self.enable_searching
	}

	pub fn is_in_search_widget(&self) -> bool {
		if let WidgetPosition::ProcessSearch = self.current_widget_selected {
			true
		} else {
			false
		}
	}

	pub fn search_with_pid(&mut self) {
		if !self.is_in_dialog() && self.is_searching() {
			if let WidgetPosition::ProcessSearch = self.current_widget_selected {
				self.search_state.searching_pid = true;
			}
		}
	}

	pub fn search_with_name(&mut self) {
		if !self.is_in_dialog() && self.is_searching() {
			if let WidgetPosition::ProcessSearch = self.current_widget_selected {
				self.search_state.searching_pid = false;
			}
		}
	}

	pub fn get_current_search_query(&self) -> &String {
		&self.search_state.current_search_query
	}

	pub fn toggle_ignore_case(&mut self) {
		if !self.is_in_dialog() && self.is_searching() {
			if let WidgetPosition::ProcessSearch = self.current_widget_selected {
				self.search_state.toggle_ignore_case();
				self.update_regex();
				self.update_process_gui = true;
			}
		}
	}

	pub fn update_regex(&mut self) {
		self.search_state.current_regex = if self.search_state.current_search_query.is_empty() {
			BASE_REGEX.clone()
		} else {
			let mut final_regex_string = self.search_state.current_search_query.clone();

			if !self.search_state.is_searching_with_regex() {
				final_regex_string = regex::escape(&final_regex_string);
			}

			if self.search_state.is_searching_whole_word() {
				final_regex_string = format!("^{}$", final_regex_string);
			}
			if self.search_state.is_ignoring_case() {
				final_regex_string = format!("(?i){}", final_regex_string);
			}

			regex::Regex::new(&final_regex_string)
		};
		self.app_scroll_positions
			.process_scroll_state
			.previous_scroll_position = 0;
		self.app_scroll_positions
			.process_scroll_state
			.current_scroll_position = 0;
	}

	pub fn get_cursor_position(&self) -> usize {
		self.search_state.current_cursor_position
	}

	/// One of two functions allowed to run while in a dialog...
	pub fn on_enter(&mut self) {
		if self.delete_dialog_state.is_showing_dd {
			if self.delete_dialog_state.is_on_yes {
				// If within dd...
				if self.dd_err.is_none() {
					// Also ensure that we didn't just fail a dd...
					let dd_result = self.kill_highlighted_process();
					self.delete_dialog_state.is_on_yes = false;

					// Check if there was an issue... if so, inform the user.
					if let Err(dd_err) = dd_result {
						self.dd_err = Some(dd_err.to_string());
					} else {
						self.delete_dialog_state.is_showing_dd = false;
					}
				}
			} else {
				self.delete_dialog_state.is_showing_dd = false;
			}
		} else if !self.is_in_dialog() {
			// Pop-out mode.
			self.is_expanded = true;
		}
	}

	pub fn on_backspace(&mut self) {
		if let WidgetPosition::ProcessSearch = self.current_widget_selected {
			if self.search_state.current_cursor_position > 0 {
				self.search_state.current_cursor_position -= 1;
				self.search_state
					.current_search_query
					.remove(self.search_state.current_cursor_position);

				self.update_regex();
				self.update_process_gui = true;
			}
		}
	}

	pub fn get_current_regex_matcher(&self) -> &std::result::Result<regex::Regex, regex::Error> {
		&self.search_state.current_regex
	}

	pub fn on_up_key(&mut self) {
		if !self.is_in_dialog() {
			if let WidgetPosition::ProcessSearch = self.current_widget_selected {
			} else {
				self.decrement_position_count();
			}
		}
	}

	pub fn on_down_key(&mut self) {
		if !self.is_in_dialog() {
			if let WidgetPosition::ProcessSearch = self.current_widget_selected {
			} else {
				self.increment_position_count();
			}
		}
	}

	pub fn on_left_key(&mut self) {
		if !self.is_in_dialog() {
			if let WidgetPosition::ProcessSearch = self.current_widget_selected {
				if self.search_state.current_cursor_position > 0 {
					self.search_state.current_cursor_position -= 1;
				}
			}
		} else if self.delete_dialog_state.is_showing_dd && !self.delete_dialog_state.is_on_yes {
			self.delete_dialog_state.is_on_yes = true;
		}
	}

	pub fn on_right_key(&mut self) {
		if !self.is_in_dialog() {
			if let WidgetPosition::ProcessSearch = self.current_widget_selected {
				if self.search_state.current_cursor_position
					< self.search_state.current_search_query.len()
				{
					self.search_state.current_cursor_position += 1;
				}
			}
		} else if self.delete_dialog_state.is_showing_dd && self.delete_dialog_state.is_on_yes {
			self.delete_dialog_state.is_on_yes = false;
		}
	}

	pub fn skip_cursor_beginning(&mut self) {
		if !self.is_in_dialog() {
			if let WidgetPosition::ProcessSearch = self.current_widget_selected {
				self.search_state.current_cursor_position = 0;
			}
		}
	}

	pub fn skip_cursor_end(&mut self) {
		if !self.is_in_dialog() {
			if let WidgetPosition::ProcessSearch = self.current_widget_selected {
				self.search_state.current_cursor_position =
					self.search_state.current_search_query.len();
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

			if let WidgetPosition::ProcessSearch = self.current_widget_selected {
				self.search_state
					.current_search_query
					.insert(self.search_state.current_cursor_position, caught_char);
				self.search_state.current_cursor_position += 1;

				self.update_regex();

				self.update_process_gui = true;
			} else {
				match caught_char {
					'/' => {
						self.enable_searching();
					}
					'd' => {
						if let WidgetPosition::Process = self.current_widget_selected {
							let mut is_first_d = true;
							if let Some(second_char) = self.second_char {
								if self.awaiting_second_char && second_char == 'd' {
									is_first_d = false;
									self.awaiting_second_char = false;
									self.second_char = None;

									if self
										.app_scroll_positions
										.process_scroll_state
										.current_scroll_position < self
										.canvas_data
										.finalized_process_data
										.len() as u64
									{
										let current_process = if self.is_grouped() {
											let group_pids = &self
												.canvas_data
												.finalized_process_data[self
												.app_scroll_positions
												.process_scroll_state
												.current_scroll_position as usize]
												.group_pids;

											let mut ret = ("".to_string(), group_pids.clone());

											for pid in group_pids {
												if let Some(process) =
													self.canvas_data.process_data.get(&pid)
												{
													ret.0 = process.name.clone();
													break;
												}
											}
											ret
										} else {
											let process = self.canvas_data.finalized_process_data
												[self
													.app_scroll_positions
													.process_scroll_state
													.current_scroll_position as usize]
												.clone();
											(process.name.clone(), vec![process.pid])
										};

										self.to_delete_process_list = Some(current_process);
										self.delete_dialog_state.is_showing_dd = true;
									}

									self.reset_multi_tap_keys();
								}
							}

							if is_first_d {
								self.awaiting_second_char = true;
								self.second_char = Some('d');
							}
						}
					}
					'g' => {
						let mut is_first_g = true;
						if let Some(second_char) = self.second_char {
							if self.awaiting_second_char && second_char == 'g' {
								is_first_g = false;
								self.awaiting_second_char = false;
								self.second_char = None;
								self.skip_to_first();
							}
						}

						if is_first_g {
							self.awaiting_second_char = true;
							self.second_char = Some('g');
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
						self.update_process_gui = true;
						self.app_scroll_positions
							.process_scroll_state
							.current_scroll_position = 0;
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
						self.update_process_gui = true;
						self.app_scroll_positions
							.process_scroll_state
							.current_scroll_position = 0;
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
							self.update_process_gui = true;
							self.app_scroll_positions
								.process_scroll_state
								.current_scroll_position = 0;
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
						self.update_process_gui = true;
						self.app_scroll_positions
							.process_scroll_state
							.current_scroll_position = 0;
					}
					'?' => {
						self.help_dialog_state.is_showing_help = true;
					}
					_ => {}
				}

				if let Some(second_char) = self.second_char {
					if self.awaiting_second_char && caught_char != second_char {
						self.awaiting_second_char = false;
					}
				}
			}
		} else if self.help_dialog_state.is_showing_help {
			match caught_char {
				'1' => self.help_dialog_state.current_category = AppHelpCategory::General,
				'2' => self.help_dialog_state.current_category = AppHelpCategory::Process,
				'3' => self.help_dialog_state.current_category = AppHelpCategory::Search,
				_ => {}
			}
		}
	}

	pub fn kill_highlighted_process(&mut self) -> Result<()> {
		// Technically unnecessary but this is a good check...
		if let WidgetPosition::Process = self.current_widget_selected {
			if let Some(current_selected_processes) = &(self.to_delete_process_list) {
				for pid in &current_selected_processes.1 {
					process_killer::kill_process_given_pid(*pid)?;
				}
			}
			self.to_delete_process_list = None;
		}
		Ok(())
	}

	pub fn get_to_delete_processes(&self) -> Option<(String, Vec<u32>)> {
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
	// PROC -(up)> Disk, -(down)> PROC_SEARCH, -(left)> Network
	// PROC_SEARCH -(up)> PROC, -(left)> Network
	pub fn move_widget_selection_left(&mut self) {
		if !self.is_in_dialog() && !self.is_expanded {
			self.current_widget_selected = match self.current_widget_selected {
				WidgetPosition::Process => WidgetPosition::Network,
				WidgetPosition::ProcessSearch => WidgetPosition::Network,
				WidgetPosition::Disk => WidgetPosition::Mem,
				WidgetPosition::Temp => WidgetPosition::Mem,
				_ => self.current_widget_selected,
			};
		}

		self.reset_multi_tap_keys();
	}

	pub fn move_widget_selection_right(&mut self) {
		if !self.is_in_dialog() && !self.is_expanded {
			self.current_widget_selected = match self.current_widget_selected {
				WidgetPosition::Mem => WidgetPosition::Temp,
				WidgetPosition::Network => WidgetPosition::Process,
				_ => self.current_widget_selected,
			};
		}

		self.reset_multi_tap_keys();
	}

	pub fn move_widget_selection_up(&mut self) {
		if !self.is_in_dialog() && !self.is_expanded {
			self.current_widget_selected = match self.current_widget_selected {
				WidgetPosition::Mem => WidgetPosition::Cpu,
				WidgetPosition::Network => WidgetPosition::Mem,
				WidgetPosition::Process => WidgetPosition::Disk,
				WidgetPosition::ProcessSearch => WidgetPosition::Process,
				WidgetPosition::Temp => WidgetPosition::Cpu,
				WidgetPosition::Disk => WidgetPosition::Temp,
				_ => self.current_widget_selected,
			};
		} else if self.is_expanded {
			self.current_widget_selected = match self.current_widget_selected {
				WidgetPosition::ProcessSearch => WidgetPosition::Process,
				_ => self.current_widget_selected,
			};
		}

		self.reset_multi_tap_keys();
	}

	pub fn move_widget_selection_down(&mut self) {
		if !self.is_in_dialog() && !self.is_expanded {
			self.current_widget_selected = match self.current_widget_selected {
				WidgetPosition::Cpu => WidgetPosition::Mem,
				WidgetPosition::Mem => WidgetPosition::Network,
				WidgetPosition::Temp => WidgetPosition::Disk,
				WidgetPosition::Disk => WidgetPosition::Process,
				WidgetPosition::Process => {
					if self.is_searching() {
						WidgetPosition::ProcessSearch
					} else {
						WidgetPosition::Process
					}
				}
				_ => self.current_widget_selected,
			};
		} else if self.is_expanded {
			self.current_widget_selected = match self.current_widget_selected {
				WidgetPosition::Process => {
					if self.is_searching() {
						WidgetPosition::ProcessSearch
					} else {
						WidgetPosition::Process
					}
				}
				_ => self.current_widget_selected,
			};
		}

		self.reset_multi_tap_keys();
	}

	pub fn skip_to_first(&mut self) {
		if !self.is_in_dialog() {
			match self.current_widget_selected {
				WidgetPosition::Process => {
					self.app_scroll_positions
						.process_scroll_state
						.current_scroll_position = 0
				}
				WidgetPosition::Temp => {
					self.app_scroll_positions
						.temp_scroll_state
						.current_scroll_position = 0
				}
				WidgetPosition::Disk => {
					self.app_scroll_positions
						.disk_scroll_state
						.current_scroll_position = 0
				}
				WidgetPosition::Cpu => {
					self.app_scroll_positions
						.cpu_scroll_state
						.current_scroll_position = 0
				}

				_ => {}
			}
			self.app_scroll_positions.scroll_direction = ScrollDirection::UP;
			self.reset_multi_tap_keys();
		}
	}

	pub fn skip_to_last(&mut self) {
		if !self.is_in_dialog() {
			match self.current_widget_selected {
				WidgetPosition::Process => {
					self.app_scroll_positions
						.process_scroll_state
						.current_scroll_position = self.canvas_data.finalized_process_data.len() as u64 - 1
				}
				WidgetPosition::Temp => {
					self.app_scroll_positions
						.temp_scroll_state
						.current_scroll_position = self.canvas_data.temp_sensor_data.len() as u64 - 1
				}
				WidgetPosition::Disk => {
					self.app_scroll_positions
						.disk_scroll_state
						.current_scroll_position = self.canvas_data.disk_data.len() as u64 - 1
				}
				WidgetPosition::Cpu => {
					self.app_scroll_positions
						.cpu_scroll_state
						.current_scroll_position = self.canvas_data.cpu_data.len() as u64 - 1;
				}
				_ => {}
			}
			self.app_scroll_positions.scroll_direction = ScrollDirection::DOWN;
			self.reset_multi_tap_keys();
		}
	}

	pub fn decrement_position_count(&mut self) {
		if !self.is_in_dialog() {
			match self.current_widget_selected {
				WidgetPosition::Process => self.change_process_position(-1),
				WidgetPosition::Temp => self.change_temp_position(-1),
				WidgetPosition::Disk => self.change_disk_position(-1),
				WidgetPosition::Cpu => self.change_cpu_table_position(-1), // TODO: [PO?] Temporary, may change if we add scaling
				_ => {}
			}
			self.app_scroll_positions.scroll_direction = ScrollDirection::UP;
			self.reset_multi_tap_keys();
		}
	}

	pub fn increment_position_count(&mut self) {
		if !self.is_in_dialog() {
			match self.current_widget_selected {
				WidgetPosition::Process => self.change_process_position(1),
				WidgetPosition::Temp => self.change_temp_position(1),
				WidgetPosition::Disk => self.change_disk_position(1),
				WidgetPosition::Cpu => self.change_cpu_table_position(1), // TODO: [PO?] Temporary, may change if we add scaling
				_ => {}
			}
			self.app_scroll_positions.scroll_direction = ScrollDirection::DOWN;
			self.reset_multi_tap_keys();
		}
	}

	fn change_cpu_table_position(&mut self, num_to_change_by: i64) {
		let current_posn = self
			.app_scroll_positions
			.cpu_scroll_state
			.current_scroll_position;

		if current_posn as i64 + num_to_change_by >= 0
			&& current_posn as i64 + num_to_change_by < self.canvas_data.cpu_data.len() as i64
		{
			self.app_scroll_positions
				.cpu_scroll_state
				.current_scroll_position = (current_posn as i64 + num_to_change_by) as u64;
		}
	}

	fn change_process_position(&mut self, num_to_change_by: i64) {
		let current_posn = self
			.app_scroll_positions
			.process_scroll_state
			.current_scroll_position;

		if current_posn as i64 + num_to_change_by >= 0
			&& current_posn as i64 + num_to_change_by
				< self.canvas_data.finalized_process_data.len() as i64
		{
			self.app_scroll_positions
				.process_scroll_state
				.current_scroll_position = (current_posn as i64 + num_to_change_by) as u64;
		}
	}

	fn change_temp_position(&mut self, num_to_change_by: i64) {
		let current_posn = self
			.app_scroll_positions
			.temp_scroll_state
			.current_scroll_position;

		if current_posn as i64 + num_to_change_by >= 0
			&& current_posn as i64 + num_to_change_by
				< self.canvas_data.temp_sensor_data.len() as i64
		{
			self.app_scroll_positions
				.temp_scroll_state
				.current_scroll_position = (current_posn as i64 + num_to_change_by) as u64;
		}
	}

	fn change_disk_position(&mut self, num_to_change_by: i64) {
		let current_posn = self
			.app_scroll_positions
			.disk_scroll_state
			.current_scroll_position;

		if current_posn as i64 + num_to_change_by >= 0
			&& current_posn as i64 + num_to_change_by < self.canvas_data.disk_data.len() as i64
		{
			self.app_scroll_positions
				.disk_scroll_state
				.current_scroll_position = (current_posn as i64 + num_to_change_by) as u64;
		}
	}
}
