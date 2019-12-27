use futures::StreamExt;
use heim::net;
use heim::units::information::byte;
use std::time::Instant;
use sysinfo::{NetworkExt, System, SystemExt};

#[derive(Debug, Clone)]
/// Note all values are in bytes...
pub struct NetworkData {
	pub rx: u64,
	pub tx: u64,
	pub total_rx: u64,
	pub total_tx: u64,
	pub instant: Instant,
}

pub async fn get_network_data(
	sys: &System, prev_net_rx_bytes: &mut u64, prev_net_tx_bytes: &mut u64, prev_net_access_time: &mut std::time::Instant,
) -> crate::utils::error::Result<NetworkData> {
	if cfg!(target_os = "windows") {
		let network_data = sys.get_network();

		*prev_net_access_time = Instant::now();
		Ok(NetworkData {
			rx: network_data.get_income(),
			tx: network_data.get_outcome(),
			total_rx: 0,
			total_tx: 0,
			instant: *prev_net_access_time,
		})
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
		let cur_time = Instant::now();
		let elapsed_time = cur_time.duration_since(*prev_net_access_time).as_secs_f64();

		let rx = ((net_rx - *prev_net_rx_bytes) as f64 / elapsed_time) as u64;
		let tx = ((net_tx - *prev_net_tx_bytes) as f64 / elapsed_time) as u64;

		*prev_net_rx_bytes = net_rx;
		*prev_net_tx_bytes = net_tx;
		*prev_net_access_time = cur_time;
		Ok(NetworkData {
			rx,
			tx,
			total_rx: *prev_net_rx_bytes,
			total_tx: *prev_net_tx_bytes,
			instant: *prev_net_access_time,
		})
	}
}
