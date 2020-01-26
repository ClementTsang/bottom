use crate::data_harvester::{mem, network, Data};
/// In charge of cleaning and managing data.  I couldn't think of a better
/// name for the file.
use std::time::Instant;
use std::vec::Vec;

pub type TimeOffset = f64;
pub type Value = f64;
pub type JoinedDataPoints = (Value, Vec<(TimeOffset, Value)>);

#[derive(Debug, Default)]
pub struct TimedData {
	pub rx_data: JoinedDataPoints,
	pub tx_data: JoinedDataPoints,
	pub cpu_data: JoinedDataPoints,
	pub mem_data: JoinedDataPoints,
	pub swap_data: JoinedDataPoints,
	pub temp_data: JoinedDataPoints,
	pub io_data: JoinedDataPoints,
}

/// AppCollection represents the pooled data stored within the main app
/// thread.  Basically stores a (occasionally cleaned) record of the data
/// collected, and what is needed to convert into a displayable form.
///
/// If the app is *frozen* - that is, we do not want to *display* any changing
/// data, keep updating this, don't convert to canvas displayable data!
///
/// Note that with this method, the *app* thread is responsible for cleaning -
/// not the data collector.
#[derive(Debug)]
pub struct DataCollection {
	pub current_instant: Instant,
	pub timed_data_vec: Vec<(Instant, TimedData)>,
	pub network_harvest: network::NetworkHarvest,
	pub memory_harvest: mem::MemHarvest,
	pub swap_harvest: mem::MemHarvest,
	// pub process_data: ProcessData,
}

impl Default for DataCollection {
	fn default() -> Self {
		DataCollection {
			current_instant: Instant::now(),
			timed_data_vec: Vec::default(),
			network_harvest: network::NetworkHarvest::default(),
			memory_harvest: mem::MemHarvest::default(),
			swap_harvest: mem::MemHarvest::default(),
			// process_data: ProcessData::default(),
		}
	}
}

impl DataCollection {
	pub fn clean_data(&mut self) {}

	pub fn eat_data(&mut self, harvested_data: &Data) {
		let harvested_time = harvested_data.last_collection_time;
		let mut new_entry = TimedData::default();

		// Network
		self.eat_network(&harvested_data, &harvested_time, &mut new_entry);

		// Memory and Swap
		self.eat_memory_and_swap(&harvested_data, &harvested_time, &mut new_entry);

		// And we're done eating.
		self.current_instant = harvested_time;
		self.timed_data_vec.push((harvested_time, new_entry));
	}

	fn eat_memory_and_swap(
		&mut self, harvested_data: &Data, harvested_time: &Instant, new_entry: &mut TimedData,
	) {
		// Memory
		let mem_percent = harvested_data.memory.mem_used_in_mb as f64
			/ harvested_data.memory.mem_total_in_mb as f64
			* 100.0;
		let mem_joining_pts = if let Some((time, last_pt)) = self.timed_data_vec.last() {
			generate_joining_points(&time, last_pt.mem_data.0, &harvested_time, mem_percent)
		} else {
			Vec::new()
		};
		let mem_pt = (mem_percent, mem_joining_pts);
		new_entry.mem_data = mem_pt;

		// Swap
		if harvested_data.swap.mem_total_in_mb > 0 {
			let swap_percent = harvested_data.swap.mem_used_in_mb as f64
				/ harvested_data.swap.mem_total_in_mb as f64
				* 100.0;
			let swap_joining_pt = if let Some((time, last_pt)) = self.timed_data_vec.last() {
				generate_joining_points(&time, last_pt.swap_data.0, &harvested_time, swap_percent)
			} else {
				Vec::new()
			};
			let swap_pt = (swap_percent, swap_joining_pt);
			new_entry.swap_data = swap_pt;
		}

		// In addition copy over latest data for easy reference
		self.memory_harvest = harvested_data.memory.clone();
		self.swap_harvest = harvested_data.swap.clone();
	}

	fn eat_network(
		&mut self, harvested_data: &Data, harvested_time: &Instant, new_entry: &mut TimedData,
	) {
		// RX
		let rx_joining_pts = if let Some((time, last_pt)) = self.timed_data_vec.last() {
			generate_joining_points(
				&time,
				last_pt.rx_data.0,
				&harvested_time,
				harvested_data.network.rx as f64,
			)
		} else {
			Vec::new()
		};
		let rx_pt = (harvested_data.network.rx as f64, rx_joining_pts);
		new_entry.rx_data = rx_pt;

		// TX
		let tx_joining_pts = if let Some((time, last_pt)) = self.timed_data_vec.last() {
			generate_joining_points(
				&time,
				last_pt.tx_data.0,
				&harvested_time,
				harvested_data.network.tx as f64,
			)
		} else {
			Vec::new()
		};
		let tx_pt = (harvested_data.network.tx as f64, tx_joining_pts);
		new_entry.tx_data = tx_pt;

		// In addition copy over latest data for easy reference
		self.network_harvest = harvested_data.network.clone();
	}
}

pub fn generate_joining_points(
	start_x: &Instant, start_y: f64, end_x: &Instant, end_y: f64,
) -> Vec<(TimeOffset, Value)> {
	let mut points: Vec<(TimeOffset, Value)> = Vec::new();

	// Convert time floats first:
	let time_difference = (*end_x).duration_since(*start_x).as_millis() as f64;
	let value_difference = end_y - start_y;

	// Let's generate... about this many points!
	let num_points = std::cmp::min(
		std::cmp::max(
			(value_difference.abs() / (time_difference + 0.0001) * 1000.0) as u64,
			100,
		),
		1000,
	);

	for itx in 0..num_points {
		points.push((
			time_difference - (itx as f64 / num_points as f64 * time_difference),
			start_y + (itx as f64 / num_points as f64 * value_difference),
		));
	}

	points
}
