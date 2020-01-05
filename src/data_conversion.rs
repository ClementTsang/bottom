use crate::{
	app::data_collection,
	constants,
	utils::gen_util::{get_exact_byte_values, get_simple_byte_values},
};
use constants::*;

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

pub fn update_temp_row(app_data: &data_collection::Data, temp_type: &data_collection::temperature::TemperatureType) -> Vec<Vec<String>> {
	let mut sensor_vector: Vec<Vec<String>> = Vec::new();

	if (&app_data.list_of_temperature_sensor).is_empty() {
		sensor_vector.push(vec!["No Sensors Found".to_string(), "".to_string()])
	} else {
		for sensor in &app_data.list_of_temperature_sensor {
			sensor_vector.push(vec![
				sensor.component_name.to_string(),
				(sensor.temperature.ceil() as u64).to_string()
					+ match temp_type {
						data_collection::temperature::TemperatureType::Celsius => "C",
						data_collection::temperature::TemperatureType::Kelvin => "K",
						data_collection::temperature::TemperatureType::Fahrenheit => "F",
					},
			]);
		}
	}

	sensor_vector
}

pub fn update_disk_row(app_data: &data_collection::Data) -> Vec<Vec<String>> {
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
						let time_difference = io_package.instant.duration_since(prev_io_package.instant).as_secs_f64();
						if io_hashmap.contains_key(trimmed_mount) && prev_io_hashmap.contains_key(trimmed_mount) {
							// Ideally change this...
							let ele = &io_hashmap[trimmed_mount];
							let prev = &prev_io_hashmap[trimmed_mount];
							let read_bytes_per_sec = ((ele.read_bytes - prev.read_bytes) as f64 / time_difference) as u64;
							let write_bytes_per_sec = ((ele.write_bytes - prev.write_bytes) as f64 / time_difference) as u64;
							let converted_read = get_simple_byte_values(read_bytes_per_sec, false);
							let converted_write = get_simple_byte_values(write_bytes_per_sec, false);
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
			format!("{:.0}%", disk.used_space as f64 / disk.total_space as f64 * 100_f64),
			format!("{:.*}{}", 0, converted_free_space.0, converted_free_space.1),
			format!("{:.*}{}", 0, converted_total_space.0, converted_total_space.1),
			io_activity.0,
			io_activity.1,
		]);
	}

	disk_vector
}

pub fn update_process_row(app_data: &data_collection::Data) -> Vec<ConvertedProcessData> {
	let mut process_vector: Vec<ConvertedProcessData> = Vec::new();

	for process in &app_data.list_of_processes {
		process_vector.push(ConvertedProcessData {
			pid: process.pid,
			name: process.command.to_string(),
			cpu_usage: format!("{:.1}%", process.cpu_usage_percent),
			mem_usage: format!(
				"{:.1}%",
				if let Some(mem_usage) = process.mem_usage_percent {
					mem_usage
				} else if let Some(mem_usage_kb) = process.mem_usage_kb {
					if let Some(mem_data) = app_data.memory.last() {
						(mem_usage_kb / 1000) as f64 / mem_data.mem_total_in_mb as f64 * 100_f64
					} else {
						0_f64
					}
				} else {
					0_f64
				}
			),
		});
	}

	process_vector
}

pub fn update_cpu_data_points(show_avg_cpu: bool, app_data: &data_collection::Data) -> Vec<ConvertedCpuData> {
	let mut cpu_data_vector: Vec<ConvertedCpuData> = Vec::new();
	let mut cpu_collection: Vec<Vec<CpuPoint>> = Vec::new();

	if !app_data.list_of_cpu_packages.is_empty() {
		// I'm sorry for the following if statement but I couldn't be bothered here...
		for cpu_num in (if show_avg_cpu { 0 } else { 1 })..app_data.list_of_cpu_packages.last().unwrap().cpu_vec.len() {
			let mut this_cpu_data: Vec<CpuPoint> = Vec::new();

			for data in &app_data.list_of_cpu_packages {
				let current_time = std::time::Instant::now();
				let current_cpu_usage = data.cpu_vec[cpu_num].cpu_usage;

				let new_entry = CpuPoint {
					time: ((TIME_STARTS_FROM as f64 - current_time.duration_since(data.instant).as_millis() as f64) * 10_f64).floor(),
					usage: current_cpu_usage,
				};

				// Now, inject our joining points...
				if let Some(previous_element_data) = this_cpu_data.last().cloned() {
					for idx in 0..50 {
						this_cpu_data.push(CpuPoint {
							time: previous_element_data.time + ((new_entry.time - previous_element_data.time) / 50.0 * f64::from(idx)),
							usage: previous_element_data.usage + ((new_entry.usage - previous_element_data.usage) / 50.0 * f64::from(idx)),
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
							&*(app_data.list_of_cpu_packages.last().unwrap().cpu_vec[i + if show_avg_cpu { 0 } else { 1 }].cpu_name)
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

pub fn update_mem_data_points(app_data: &data_collection::Data) -> Vec<(f64, f64)> {
	convert_mem_data(&app_data.memory)
}

pub fn update_swap_data_points(app_data: &data_collection::Data) -> Vec<(f64, f64)> {
	convert_mem_data(&app_data.swap)
}

pub fn update_mem_data_values(app_data: &data_collection::Data) -> Vec<(u64, u64)> {
	let mut result: Vec<(u64, u64)> = Vec::new();
	result.push(get_most_recent_mem_values(&app_data.memory));
	result.push(get_most_recent_mem_values(&app_data.swap));

	result
}

fn get_most_recent_mem_values(mem_data: &[data_collection::mem::MemData]) -> (u64, u64) {
	let mut result: (u64, u64) = (0, 0);

	if !mem_data.is_empty() {
		if let Some(most_recent) = mem_data.last() {
			result.0 = most_recent.mem_used_in_mb;
			result.1 = most_recent.mem_total_in_mb;
		}
	}

	result
}

fn convert_mem_data(mem_data: &[data_collection::mem::MemData]) -> Vec<(f64, f64)> {
	let mut result: Vec<(f64, f64)> = Vec::new();

	for data in mem_data {
		let current_time = std::time::Instant::now();
		let new_entry = (
			((TIME_STARTS_FROM as f64 - current_time.duration_since(data.instant).as_millis() as f64) * 10_f64).floor(),
			if data.mem_total_in_mb == 0 {
				-1000.0
			} else {
				(data.mem_used_in_mb as f64 * 100_f64) / data.mem_total_in_mb as f64
			},
		);

		// Now, inject our joining points...
		if !result.is_empty() {
			let previous_element_data = *(result.last().unwrap());
			for idx in 0..50 {
				result.push((
					previous_element_data.0 + ((new_entry.0 - previous_element_data.0) / 50.0 * f64::from(idx)),
					previous_element_data.1 + ((new_entry.1 - previous_element_data.1) / 50.0 * f64::from(idx)),
				));
			}
		}

		result.push(new_entry);
	}

	result
}

pub fn update_network_data_points(app_data: &data_collection::Data) -> ConvertedNetworkData {
	convert_network_data_points(&app_data.network)
}

pub fn convert_network_data_points(network_data: &[data_collection::network::NetworkData]) -> ConvertedNetworkData {
	let mut rx: Vec<(f64, f64)> = Vec::new();
	let mut tx: Vec<(f64, f64)> = Vec::new();

	for data in network_data {
		let current_time = std::time::Instant::now();
		let rx_data = (
			((TIME_STARTS_FROM as f64 - current_time.duration_since(data.instant).as_millis() as f64) * 10_f64).floor(),
			if data.rx > 0 { (data.rx as f64).log(2.0) } else { 0.0 },
		);
		let tx_data = (
			((TIME_STARTS_FROM as f64 - current_time.duration_since(data.instant).as_millis() as f64) * 10_f64).floor(),
			if data.tx > 0 { (data.tx as f64).log(2.0) } else { 0.0 },
		);

		//debug!("Plotting: {:?} bytes rx, {:?} bytes tx", rx_data, tx_data);

		// Now, inject our joining points...
		if !rx.is_empty() {
			let previous_element_data = *(rx.last().unwrap());
			for idx in 0..50 {
				rx.push((
					previous_element_data.0 + ((rx_data.0 - previous_element_data.0) / 50.0 * f64::from(idx)),
					previous_element_data.1 + ((rx_data.1 - previous_element_data.1) / 50.0 * f64::from(idx)),
				));
			}
		}

		// Now, inject our joining points...
		if !tx.is_empty() {
			let previous_element_data = *(tx.last().unwrap());
			for idx in 0..50 {
				tx.push((
					previous_element_data.0 + ((tx_data.0 - previous_element_data.0) / 50.0 * f64::from(idx)),
					previous_element_data.1 + ((tx_data.1 - previous_element_data.1) / 50.0 * f64::from(idx)),
				));
			}
		}

		rx.push(rx_data);
		tx.push(tx_data);
	}

	let total_rx_converted_result: (f64, String);
	let rx_converted_result: (f64, String);
	let total_tx_converted_result: (f64, String);
	let tx_converted_result: (f64, String);

	if let Some(last_num_bytes_entry) = network_data.last() {
		rx_converted_result = get_exact_byte_values(last_num_bytes_entry.rx, false);
		total_rx_converted_result = get_exact_byte_values(last_num_bytes_entry.total_rx, false)
	} else {
		rx_converted_result = get_exact_byte_values(0, false);
		total_rx_converted_result = get_exact_byte_values(0, false);
	}
	let rx_display = format!("{:.*}{}", 1, rx_converted_result.0, rx_converted_result.1);
	let total_rx_display = if cfg!(not(target_os = "windows")) {
		format!("{:.*}{}", 1, total_rx_converted_result.0, total_rx_converted_result.1)
	} else {
		"N/A".to_string()
	};

	if let Some(last_num_bytes_entry) = network_data.last() {
		tx_converted_result = get_exact_byte_values(last_num_bytes_entry.tx, false);
		total_tx_converted_result = get_exact_byte_values(last_num_bytes_entry.total_tx, false);
	} else {
		tx_converted_result = get_exact_byte_values(0, false);
		total_tx_converted_result = get_exact_byte_values(0, false);
	}
	let tx_display = format!("{:.*}{}", 1, tx_converted_result.0, tx_converted_result.1);
	let total_tx_display = if cfg!(not(target_os = "windows")) {
		format!("{:.*}{}", 1, total_tx_converted_result.0, total_tx_converted_result.1)
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
