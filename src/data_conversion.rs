//! This mainly concerns converting collected data into things that the canvas
//! can actually handle.

use crate::{
	app::data_harvester,
	app::data_janitor,
	constants,
	utils::gen_util::{get_exact_byte_values, get_simple_byte_values},
};
use constants::*;
use regex::Regex;
use std::time::Instant;

#[derive(Default, Debug)]
pub struct ConvertedNetworkData {
	pub rx: Vec<(f64, f64)>,
	pub tx: Vec<(f64, f64)>,
	pub rx_display: String,
	pub tx_display: String,
	pub total_rx_display: String,
	pub total_tx_display: String,
}

#[derive(Clone, Default, Debug)]
pub struct ConvertedProcessData {
	pub pid: u32,
	pub name: String,
	pub cpu_usage: String,
	pub mem_usage: String,
	pub group: Vec<u32>,
}

#[derive(Clone, Default, Debug)]
pub struct ConvertedCpuData {
	pub cpu_name: String,
	pub cpu_data: Vec<CpuPoint>,
}

#[derive(Clone, Default, Debug)]
pub struct CpuPoint {
	pub time: f64,
	pub usage: f64,
}

impl From<CpuPoint> for (f64, f64) {
	fn from(c: CpuPoint) -> (f64, f64) {
		let CpuPoint { time, usage } = c;
		(time, usage)
	}
}

impl From<&CpuPoint> for (f64, f64) {
	fn from(c: &CpuPoint) -> (f64, f64) {
		let CpuPoint { time, usage } = c;
		(*time, *usage)
	}
}

pub fn update_temp_row(
	app_data: &data_harvester::Data, temp_type: &data_harvester::temperature::TemperatureType,
) -> Vec<Vec<String>> {
	let mut sensor_vector: Vec<Vec<String>> = Vec::new();

	if (&app_data.list_of_temperature_sensor).is_empty() {
		sensor_vector.push(vec!["No Sensors Found".to_string(), "".to_string()])
	} else {
		for sensor in &app_data.list_of_temperature_sensor {
			sensor_vector.push(vec![
				sensor.component_name.to_string(),
				(sensor.temperature.ceil() as u64).to_string()
					+ match temp_type {
						data_harvester::temperature::TemperatureType::Celsius => "C",
						data_harvester::temperature::TemperatureType::Kelvin => "K",
						data_harvester::temperature::TemperatureType::Fahrenheit => "F",
					},
			]);
		}
	}

	sensor_vector
}

pub fn update_disk_row(app_data: &data_harvester::Data) -> Vec<Vec<String>> {
	let mut disk_vector: Vec<Vec<String>> = Vec::new();
	for disk in &app_data.list_of_disks {
		let io_activity = {
			let mut final_result = ("0B/s".to_string(), "0B/s".to_string());
			if app_data.list_of_io.len() > 2 {
				if let Some(io_package) = &app_data.list_of_io.last() {
					if let Some(trimmed_mount) = disk.name.to_string().split('/').last() {
						let prev_io_package = &app_data.list_of_io[app_data.list_of_io.len() - 2];

						let io_hashmap = &io_package.io_hash;
						let prev_io_hashmap = &prev_io_package.io_hash;
						let time_difference = io_package
							.instant
							.duration_since(prev_io_package.instant)
							.as_secs_f64();
						if io_hashmap.contains_key(trimmed_mount)
							&& prev_io_hashmap.contains_key(trimmed_mount)
						{
							// Ideally change this...
							let ele = &io_hashmap[trimmed_mount];
							let prev = &prev_io_hashmap[trimmed_mount];
							let read_bytes_per_sec = ((ele.read_bytes - prev.read_bytes) as f64
								/ time_difference) as u64;
							let write_bytes_per_sec = ((ele.write_bytes - prev.write_bytes) as f64
								/ time_difference) as u64;
							let converted_read = get_simple_byte_values(read_bytes_per_sec, false);
							let converted_write =
								get_simple_byte_values(write_bytes_per_sec, false);
							final_result = (
								format!("{:.*}{}/s", 0, converted_read.0, converted_read.1),
								format!("{:.*}{}/s", 0, converted_write.0, converted_write.1),
							);
						}
					}
				}
			}
			final_result
		};

		let converted_free_space = get_simple_byte_values(disk.free_space, false);
		let converted_total_space = get_simple_byte_values(disk.total_space, false);
		disk_vector.push(vec![
			disk.name.to_string(),
			disk.mount_point.to_string(),
			format!(
				"{:.0}%",
				disk.used_space as f64 / disk.total_space as f64 * 100_f64
			),
			format!("{:.*}{}", 0, converted_free_space.0, converted_free_space.1),
			format!(
				"{:.*}{}",
				0, converted_total_space.0, converted_total_space.1
			),
			io_activity.0,
			io_activity.1,
		]);
	}

	disk_vector
}

pub fn simple_update_process_row(
	app_data: &data_harvester::Data, matching_string: &str, use_pid: bool,
) -> (Vec<ConvertedProcessData>, Vec<ConvertedProcessData>) {
	let process_vector: Vec<ConvertedProcessData> = app_data
		.list_of_processes
		.iter()
		.filter(|process| {
			if use_pid {
				process
					.pid
					.to_string()
					.to_ascii_lowercase()
					.contains(matching_string)
			} else {
				process.name.to_ascii_lowercase().contains(matching_string)
			}
		})
		.map(|process| return_mapped_process(process))
		.collect::<Vec<_>>();

	let mut grouped_process_vector: Vec<ConvertedProcessData> = Vec::new();
	if let Some(grouped_list_of_processes) = &app_data.grouped_list_of_processes {
		grouped_process_vector = grouped_list_of_processes
			.iter()
			.filter(|process| {
				if use_pid {
					process
						.pid
						.to_string()
						.to_ascii_lowercase()
						.contains(matching_string)
				} else {
					process.name.to_ascii_lowercase().contains(matching_string)
				}
			})
			.map(|process| return_mapped_process(process))
			.collect::<Vec<_>>();
	}

	(process_vector, grouped_process_vector)
}

pub fn regex_update_process_row(
	app_data: &data_harvester::Data, regex_matcher: &std::result::Result<Regex, regex::Error>,
	use_pid: bool,
) -> (Vec<ConvertedProcessData>, Vec<ConvertedProcessData>) {
	let process_vector: Vec<ConvertedProcessData> = app_data
		.list_of_processes
		.iter()
		.filter(|process| {
			if let Ok(matcher) = regex_matcher {
				if use_pid {
					matcher.is_match(&process.pid.to_string())
				} else {
					matcher.is_match(&process.name)
				}
			} else {
				true
			}
		})
		.map(|process| return_mapped_process(process))
		.collect::<Vec<_>>();

	let mut grouped_process_vector: Vec<ConvertedProcessData> = Vec::new();
	if let Some(grouped_list_of_processes) = &app_data.grouped_list_of_processes {
		grouped_process_vector = grouped_list_of_processes
			.iter()
			.filter(|process| {
				if let Ok(matcher) = regex_matcher {
					if use_pid {
						matcher.is_match(&process.pid.to_string())
					} else {
						matcher.is_match(&process.name)
					}
				} else {
					true
				}
			})
			.map(|process| return_mapped_process(process))
			.collect::<Vec<_>>();
	}

	(process_vector, grouped_process_vector)
}

fn return_mapped_process(process: &data_harvester::processes::ProcessData) -> ConvertedProcessData {
	ConvertedProcessData {
		pid: process.pid,
		name: process.name.to_string(),
		cpu_usage: format!("{:.1}%", process.cpu_usage_percent),
		mem_usage: format!("{:.1}%", process.mem_usage_percent),
		group: vec![],
	}
}

pub fn update_cpu_data_points(
	show_avg_cpu: bool, app_data: &data_harvester::Data,
) -> Vec<ConvertedCpuData> {
	let mut cpu_data_vector: Vec<ConvertedCpuData> = Vec::new();
	let mut cpu_collection: Vec<Vec<CpuPoint>> = Vec::new();

	if !app_data.list_of_cpu_packages.is_empty() {
		// I'm sorry for the following if statement but I couldn't be bothered here...
		for cpu_num in (if show_avg_cpu { 0 } else { 1 })
			..app_data.list_of_cpu_packages.last().unwrap().cpu_vec.len()
		{
			let mut this_cpu_data: Vec<CpuPoint> = Vec::new();

			for data in &app_data.list_of_cpu_packages {
				let current_time = Instant::now();
				let current_cpu_usage = data.cpu_vec[cpu_num].cpu_usage;

				let new_entry = CpuPoint {
					time: ((TIME_STARTS_FROM as f64
						- current_time.duration_since(data.instant).as_millis() as f64)
						* 10_f64)
						.floor(),
					usage: current_cpu_usage,
				};

				// Now, inject our joining points...
				if let Some(previous_element_data) = this_cpu_data.last().cloned() {
					for idx in 0..50 {
						this_cpu_data.push(CpuPoint {
							time: previous_element_data.time
								+ ((new_entry.time - previous_element_data.time) / 50.0
									* f64::from(idx)),
							usage: previous_element_data.usage
								+ ((new_entry.usage - previous_element_data.usage) / 50.0
									* f64::from(idx)),
						});
					}
				}

				this_cpu_data.push(new_entry);
			}

			cpu_collection.push(this_cpu_data);
		}

		// Finally, add it all onto the end
		for (i, data) in cpu_collection.iter().enumerate() {
			if !app_data.list_of_cpu_packages.is_empty() {
				// Commented out: this version includes the percentage in the label...
				// cpu_data_vector.push((
				// 	// + 1 to skip total CPU if show_avg_cpu is false
				// 	format!(
				// 		"{:4}: ",
				// 		&*(app_data.list_of_cpu_packages.last().unwrap().cpu_vec[i + if show_avg_cpu { 0 } else { 1 }].cpu_name)
				// 	)
				// 	.to_uppercase() + &format!("{:3}%", (data.last().unwrap_or(&(0_f64, 0_f64)).1.round() as u64)),
				// 	data.clone(),
				// ))
				cpu_data_vector.push(ConvertedCpuData {
					cpu_name: format!(
						"{} ",
						if show_avg_cpu && i == 0 {
							"AVG"
						} else {
							&*(app_data.list_of_cpu_packages.last().unwrap().cpu_vec
								[i + if show_avg_cpu { 0 } else { 1 }]
							.cpu_name)
						}
					)
					.to_uppercase(),
					cpu_data: data.clone(),
				});
			}
		}
	}

	cpu_data_vector
}

pub fn update_mem_data_points(current_data: &data_janitor::DataCollection) -> Vec<(f64, f64)> {
	let mut result: Vec<(f64, f64)> = Vec::new();
	let current_time = current_data.current_instant;

	for (time, data) in &current_data.timed_data_vec {
		let time_from_start: f64 = (TIME_STARTS_FROM as f64
			- current_time.duration_since(*time).as_millis() as f64)
			.floor();

		//Insert joiner points
		for &(joiner_offset, joiner_val) in &data.mem_data.1 {
			let offset_time = time_from_start - joiner_offset as f64;
			result.push((offset_time, joiner_val));
		}

		result.push((time_from_start, data.mem_data.0));
	}

	result
}

pub fn update_swap_data_points(current_data: &data_janitor::DataCollection) -> Vec<(f64, f64)> {
	let mut result: Vec<(f64, f64)> = Vec::new();
	let current_time = current_data.current_instant;

	for (time, data) in &current_data.timed_data_vec {
		let time_from_start: f64 = (TIME_STARTS_FROM as f64
			- current_time.duration_since(*time).as_millis() as f64)
			.floor();

		//Insert joiner points
		for &(joiner_offset, joiner_val) in &data.swap_data.1 {
			let offset_time = time_from_start - joiner_offset as f64;
			result.push((offset_time, joiner_val));
		}

		result.push((time_from_start, data.swap_data.0));
	}

	result
}

pub fn update_mem_labels(current_data: &data_janitor::DataCollection) -> (String, String) {
	let mem_label = if current_data.memory_harvest.mem_total_in_mb == 0 {
		"".to_string()
	} else {
		"RAM:".to_string()
			+ &format!(
				"{:3.0}%",
				(current_data.memory_harvest.mem_used_in_mb as f64 * 100.0
					/ current_data.memory_harvest.mem_total_in_mb as f64)
					.round()
			) + &format!(
			"   {:.1}GB/{:.1}GB",
			current_data.memory_harvest.mem_used_in_mb as f64 / 1024.0,
			current_data.memory_harvest.mem_total_in_mb as f64 / 1024.0
		)
	};

	let swap_label = if current_data.swap_harvest.mem_total_in_mb == 0 {
		"".to_string()
	} else {
		"SWP:".to_string()
			+ &format!(
				"{:3.0}%",
				(current_data.swap_harvest.mem_used_in_mb as f64 * 100.0
					/ current_data.swap_harvest.mem_total_in_mb as f64)
					.round()
			) + &format!(
			"   {:.1}GB/{:.1}GB",
			current_data.swap_harvest.mem_used_in_mb as f64 / 1024.0,
			current_data.swap_harvest.mem_total_in_mb as f64 / 1024.0
		)
	};

	debug!("{:?}", mem_label);

	(mem_label, swap_label)
}

pub fn convert_network_data_points(
	current_data: &data_janitor::DataCollection,
) -> ConvertedNetworkData {
	let mut rx: Vec<(f64, f64)> = Vec::new();
	let mut tx: Vec<(f64, f64)> = Vec::new();

	let current_time = current_data.current_instant;
	for (time, data) in &current_data.timed_data_vec {
		let time_from_start: f64 = (TIME_STARTS_FROM as f64
			- current_time.duration_since(*time).as_millis() as f64)
			.floor();

		//Insert joiner points
		for &(joiner_offset, joiner_val) in &data.rx_data.1 {
			let offset_time = time_from_start - joiner_offset as f64;
			rx.push((
				offset_time,
				if joiner_val > 0.0 {
					(joiner_val).log(2.0)
				} else {
					0.0
				},
			));
		}

		for &(joiner_offset, joiner_val) in &data.tx_data.1 {
			let offset_time = time_from_start - joiner_offset as f64;
			tx.push((
				offset_time,
				if joiner_val > 0.0 {
					(joiner_val).log(2.0)
				} else {
					0.0
				},
			));
		}

		rx.push((
			time_from_start,
			if data.rx_data.0 > 0.0 {
				(data.rx_data.0).log(2.0)
			} else {
				0.0
			},
		));
		tx.push((
			time_from_start,
			if data.rx_data.0 > 0.0 {
				(data.rx_data.0).log(2.0)
			} else {
				0.0
			},
		));
	}

	let total_rx_converted_result: (f64, String);
	let rx_converted_result: (f64, String);
	let total_tx_converted_result: (f64, String);
	let tx_converted_result: (f64, String);

	rx_converted_result = get_exact_byte_values(current_data.network_harvest.rx, false);
	total_rx_converted_result = get_exact_byte_values(current_data.network_harvest.total_rx, false);
	let rx_display = format!("{:.*}{}", 1, rx_converted_result.0, rx_converted_result.1);
	let total_rx_display = if cfg!(not(target_os = "windows")) {
		format!(
			"{:.*}{}",
			1, total_rx_converted_result.0, total_rx_converted_result.1
		)
	} else {
		"N/A".to_string()
	};

	tx_converted_result = get_exact_byte_values(current_data.network_harvest.tx, false);
	total_tx_converted_result = get_exact_byte_values(current_data.network_harvest.total_tx, false);
	let tx_display = format!("{:.*}{}", 1, tx_converted_result.0, tx_converted_result.1);
	let total_tx_display = if cfg!(not(target_os = "windows")) {
		format!(
			"{:.*}{}",
			1, total_tx_converted_result.0, total_tx_converted_result.1
		)
	} else {
		"N/A".to_string()
	};

	ConvertedNetworkData {
		rx,
		tx,
		rx_display,
		tx_display,
		total_rx_display,
		total_tx_display,
	}
}
