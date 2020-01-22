use futures::StreamExt;
use heim::net;
use heim::units::information::byte;
use std::time::Instant;
use sysinfo::{NetworkExt, System, SystemExt};

#[derive(Clone, Debug)]
pub struct NetworkJoinPoint {
	pub rx: f64,
	pub tx: f64,
	pub time_offset_milliseconds: f64,
}

type NetworkDataGroup = (Instant, (NetworkData, Option<Vec<NetworkJoinPoint>>));
#[derive(Clone, Debug)]
pub struct NetworkStorage {
	pub data_points: Vec<NetworkDataGroup>,
	pub rx: u64,
	pub tx: u64,
	pub total_rx: u64,
	pub total_tx: u64,
	pub last_collection_time: Instant,
}

impl Default for NetworkStorage {
	fn default() -> Self {
		NetworkStorage {
			data_points: Vec::default(),
			rx: 0,
			tx: 0,
			total_rx: 0,
			total_tx: 0,
			last_collection_time: Instant::now(),
		}
	}
}

impl NetworkStorage {
	pub fn first_run(&mut self) {
		self.data_points = Vec::default();
		self.rx = 0;
		self.tx = 0;
	}
}

#[derive(Clone, Debug)]
/// Note all values are in bytes...
pub struct NetworkData {
	pub rx: u64,
	pub tx: u64,
}

pub async fn get_network_data(
	sys: &System, prev_net_access_time: &Instant, prev_net_rx: &mut u64, prev_net_tx: &mut u64,
	curr_time: &Instant,
) -> NetworkData {
	// FIXME: [WIN] Track current total bytes... also is this accurate?
	if cfg!(target_os = "windows") {
		let network_data = sys.get_network();
		NetworkData {
			rx: network_data.get_income(),
			tx: network_data.get_outcome(),
		}
	} else {
		let mut io_data = net::io_counters();
		let mut net_rx: u64 = 0;
		let mut net_tx: u64 = 0;

		while let Some(io) = io_data.next().await {
			if let Ok(io) = io {
				net_rx += io.bytes_recv().get::<byte>();
				net_tx += io.bytes_sent().get::<byte>();
			}
		}
		let elapsed_time = curr_time
			.duration_since(*prev_net_access_time)
			.as_secs_f64();

		let rx = ((net_rx - *prev_net_rx) as f64 / elapsed_time) as u64;
		let tx = ((net_tx - *prev_net_tx) as f64 / elapsed_time) as u64;

		*prev_net_rx = net_rx;
		*prev_net_tx = net_tx;
		NetworkData { rx, tx }
	}
}
