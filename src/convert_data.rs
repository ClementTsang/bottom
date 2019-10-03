use crate::{app::data_collection, constants};
use constants::*;

pub fn update_temp_row(app_data : &data_collection::Data, temp_type : &data_collection::temperature::TemperatureType) -> Vec<Vec<String>> {
	let mut sensor_vector : Vec<Vec<String>> = Vec::new();

	if (&app_data.list_of_temperature_sensor).is_empty() {
		sensor_vector.push(vec!["No Sensors Found".to_string(), "".to_string()])
	}
	else {
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

pub fn update_disk_row(app_data : &data_collection::Data) -> Vec<Vec<String>> {
	let mut disk_vector : Vec<Vec<String>> = Vec::new();
	for disk in &app_data.list_of_disks {
		let io_activity = if app_data.list_of_io.len() > 2 {
			let io_package = &app_data.list_of_io.last().unwrap();
			let prev_io_package = &app_data.list_of_io[app_data.list_of_io.len() - 2];

			let io_hashmap = &io_package.io_hash;
			let prev_io_hashmap = &prev_io_package.io_hash;
			let trimmed_mount = &disk.name.to_string().split('/').last().unwrap().to_string();
			let time_difference = io_package.instant.duration_since(prev_io_package.instant).as_secs_f64();
			if io_hashmap.contains_key(trimmed_mount) && prev_io_hashmap.contains_key(trimmed_mount) {
				// Ideally change this...
				let ele = &io_hashmap[trimmed_mount];
				let prev = &prev_io_hashmap[trimmed_mount];
				let read_bytes_per_sec = ((ele.read_bytes - prev.read_bytes) as f64 / time_difference) as u64;
				let write_bytes_per_sec = ((ele.write_bytes - prev.write_bytes) as f64 / time_difference) as u64;
				(
					if read_bytes_per_sec < 1024 {
						format!("{}B", read_bytes_per_sec)
					}
					else if read_bytes_per_sec < 1024 * 1024 {
						format!("{}KB", read_bytes_per_sec / 1024)
					}
					else {
						format!("{}MB", read_bytes_per_sec / 1024 / 1024)
					},
					if write_bytes_per_sec < 1024 {
						format!("{}B", write_bytes_per_sec)
					}
					else if write_bytes_per_sec < 1024 * 1024 {
						format!("{}KB", write_bytes_per_sec / 1024)
					}
					else {
						format!("{}MB", write_bytes_per_sec / 1024 / 1024)
					},
				)
			}
			else {
				("0B".to_string(), "0B".to_string())
			}
		}
		else {
			("0B".to_string(), "0B".to_string())
		};
		disk_vector.push(vec![
			disk.name.to_string(),
			disk.mount_point.to_string(),
			format!("{:.0}%", disk.used_space as f64 / disk.total_space as f64 * 100_f64),
			if disk.free_space < 1024 {
				disk.free_space.to_string() + "MB"
			}
			else {
				(disk.free_space / 1024).to_string() + "GB"
			},
			if disk.total_space < 1024 {
				disk.total_space.to_string() + "MB"
			}
			else {
				(disk.total_space / 1024).to_string() + "GB"
			},
			io_activity.0,
			io_activity.1,
		]);
	}

	disk_vector
}

pub fn update_process_row(app_data : &data_collection::Data) -> Vec<Vec<String>> {
	let mut process_vector : Vec<Vec<String>> = Vec::new();

	for process in &app_data.list_of_processes {
		process_vector.push(vec![
			process.pid.to_string(),
			process.command.to_string(),
			format!("{:.1}%", process.cpu_usage_percent),
			format!(
				"{:.1}%",
				if let Some(mem_usage) = process.mem_usage_percent {
					mem_usage
				}
				else if let Some(mem_usage_kb) = process.mem_usage_kb {
					if let Some(mem_data) = app_data.memory.last() {
						(mem_usage_kb / 1024) as f64 / mem_data.mem_total_in_mb as f64 * 100_f64
					}
					else {
						0_f64
					}
				}
				else {
					0_f64
				}
			),
		]);
	}

	process_vector
}

pub fn update_cpu_data_points(show_avg_cpu : bool, app_data : &data_collection::Data) -> Vec<(String, Vec<(f64, f64)>)> {
	let mut cpu_data_vector : Vec<(String, Vec<(f64, f64)>)> = Vec::new();
	let mut cpu_collection : Vec<Vec<(f64, f64)>> = Vec::new();

	if !app_data.list_of_cpu_packages.is_empty() {
		// I'm sorry for the if statement but I couldn't be bothered here...
		for cpu_num in (if show_avg_cpu { 0 } else { 1 })..app_data.list_of_cpu_packages.last().unwrap().cpu_vec.len() {
			let mut this_cpu_data : Vec<(f64, f64)> = Vec::new();

			for data in &app_data.list_of_cpu_packages {
				let current_time = std::time::Instant::now();
				let current_cpu_usage = data.cpu_vec[cpu_num].cpu_usage;

				let new_entry = (
					((STALE_MAX_MILLISECONDS as f64 - current_time.duration_since(data.instant).as_millis() as f64) * 10_f64).floor(),
					current_cpu_usage,
				);

				// Now, inject our joining points...
				if !this_cpu_data.is_empty() {
					let previous_element_data = *(this_cpu_data.last().unwrap());
					for idx in 0..50 {
						this_cpu_data.push((
							previous_element_data.0 + ((new_entry.0 - previous_element_data.0) / 50.0 * f64::from(idx)),
							previous_element_data.1 + ((new_entry.1 - previous_element_data.1) / 50.0 * f64::from(idx)),
						));
					}
				}

				this_cpu_data.push(new_entry);
			}

			cpu_collection.push(this_cpu_data);
		}

		// Finally, add it all onto the end
		for (i, data) in cpu_collection.iter().enumerate() {
			cpu_data_vector.push((
				// + 1 to skip total CPU if show_avg_cpu is false
				format!(
					"{:4}: ",
					&*(app_data.list_of_cpu_packages.last().unwrap().cpu_vec[i + if show_avg_cpu { 0 } else { 1 }].cpu_name)
				)
				.to_uppercase() + &format!("{:3}%", (data.last().unwrap_or(&(0_f64, 0_f64)).1.round() as u64)),
				data.clone(),
			))
		}
	}

	cpu_data_vector
}

pub fn update_mem_data_points(app_data : &data_collection::Data) -> Vec<(f64, f64)> {
	convert_mem_data(&app_data.memory)
}

pub fn update_swap_data_points(app_data : &data_collection::Data) -> Vec<(f64, f64)> {
	convert_mem_data(&app_data.swap)
}

pub fn update_mem_data_values(app_data : &data_collection::Data) -> Vec<(u64, u64)> {
	let mut result : Vec<(u64, u64)> = Vec::new();
	result.push(get_most_recent_mem_values(&app_data.memory));
	result.push(get_most_recent_mem_values(&app_data.swap));

	result
}

fn get_most_recent_mem_values(mem_data : &[data_collection::mem::MemData]) -> (u64, u64) {
	let mut result : (u64, u64) = (0, 0);

	if !mem_data.is_empty() {
		if let Some(most_recent) = mem_data.last() {
			result.0 = most_recent.mem_used_in_mb;
			result.1 = most_recent.mem_total_in_mb;
		}
	}

	result
}

fn convert_mem_data(mem_data : &[data_collection::mem::MemData]) -> Vec<(f64, f64)> {
	let mut result : Vec<(f64, f64)> = Vec::new();

	for data in mem_data {
		let current_time = std::time::Instant::now();
		let new_entry = (
			((STALE_MAX_MILLISECONDS as f64 - current_time.duration_since(data.instant).as_millis() as f64) * 10_f64).floor(),
			if data.mem_total_in_mb == 0 {
				-1000.0
			}
			else {
				data.mem_used_in_mb as f64 / data.mem_total_in_mb as f64 * 100_f64
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

pub struct ConvertedNetworkData {
	pub rx : Vec<(f64, f64)>,
	pub tx : Vec<(f64, f64)>,
	pub rx_display : String,
	pub tx_display : String,
}

pub fn update_network_data_points(app_data : &data_collection::Data) -> ConvertedNetworkData {
	convert_network_data_points(&app_data.network)
}

pub fn convert_network_data_points(network_data : &[data_collection::network::NetworkData]) -> ConvertedNetworkData {
	let mut rx : Vec<(f64, f64)> = Vec::new();
	let mut tx : Vec<(f64, f64)> = Vec::new();

	for data in network_data {
		let current_time = std::time::Instant::now();
		let rx_data = (
			((STALE_MAX_MILLISECONDS as f64 - current_time.duration_since(data.instant).as_millis() as f64) * 10_f64).floor(),
			data.rx as f64 / 1024.0,
		);
		let tx_data = (
			((STALE_MAX_MILLISECONDS as f64 - current_time.duration_since(data.instant).as_millis() as f64) * 10_f64).floor(),
			data.tx as f64 / 1024.0,
		);

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

		//debug!("Pushed rx: ({}, {})", rx.last().unwrap().0, rx.last().unwrap().1);
		//debug!("Pushed tx: ({}, {})", tx.last().unwrap().0, tx.last().unwrap().1);
	}

	let rx_display = if network_data.is_empty() {
		"0B".to_string()
	}
	else {
		let num_bytes = network_data.last().unwrap().rx;
		if num_bytes < 1024 {
			format!("RX: {:4} B", num_bytes).to_string()
		}
		else if num_bytes < (1024 * 1024) {
			format!("RX: {:4}KB", num_bytes / 1024).to_string()
		}
		else if num_bytes < (1024 * 1024 * 1024) {
			format!("RX: {:4}MB", num_bytes / 1024 / 1024).to_string()
		}
		else {
			format!("RX: {:4}GB", num_bytes / 1024 / 1024 / 1024).to_string()
		}
	};
	let tx_display = if network_data.is_empty() {
		"0B".to_string()
	}
	else {
		let num_bytes = network_data.last().unwrap().tx;
		if num_bytes < 1024 {
			format!("TX: {:4} B", num_bytes).to_string()
		}
		else if num_bytes < (1024 * 1024) {
			format!("TX: {:4}KB", num_bytes / 1024).to_string()
		}
		else if num_bytes < (1024 * 1024 * 1024) {
			format!("TX: {:4}MB", num_bytes / 1024 / 1024).to_string()
		}
		else {
			format!("TX: {:4}GB", num_bytes / 1024 / 1024 / 1024).to_string()
		}
	};

	ConvertedNetworkData {
		rx,
		tx,
		rx_display,
		tx_display,
	}
}
