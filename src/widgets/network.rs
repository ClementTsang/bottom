pub struct TimedNetworkData {
	pub rx : u32,
	pub tx : u32,
	pub time : std::time::SystemTime,
}

pub fn get_network_data() -> TimedNetworkData {
	TimedNetworkData {
		rx : 0,
		tx : 0,
		time : std::time::SystemTime::now(),
	}
}
