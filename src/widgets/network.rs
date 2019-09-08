use sysinfo::{NetworkExt, System, SystemExt};

#[derive(Clone, Default)]
pub struct NetworkData {
	pub rx : u64,
	pub tx : u64,
}

pub fn get_network_data(sys : &System) -> Result<NetworkData, heim::Error> {
	let network_data = sys.get_network();
	Ok(NetworkData {
		rx : network_data.get_income(),
		tx : network_data.get_outcome(),
	})
}
