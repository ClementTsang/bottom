use sysinfo::{NetworkExt, System, SystemExt};

pub struct TimedNetworkData {
	pub rx : u64,
	pub tx : u64,
	pub time : std::time::SystemTime,
}

pub fn get_network_data(sys : &System) -> TimedNetworkData {
	let network_data = sys.get_network();
	TimedNetworkData {
		rx : network_data.get_income(),
		tx : network_data.get_outcome(),
		time : std::time::SystemTime::now(),
	}
}
