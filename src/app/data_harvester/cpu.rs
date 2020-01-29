use sysinfo::{ProcessorExt, System, SystemExt};

#[derive(Default, Debug, Clone)]
pub struct CPUData {
	pub cpu_name: String,
	pub cpu_usage: f64,
}

pub type CPUHarvest = Vec<CPUData>;

pub fn get_cpu_data_list(sys: &System) -> CPUHarvest {
	let cpu_data = sys.get_processor_list();
	let mut cpu_vec = Vec::new();

	for cpu in cpu_data {
		cpu_vec.push(CPUData {
			cpu_name: cpu.get_name().to_string(),
			cpu_usage: f64::from(cpu.get_cpu_usage()) * 100_f64,
		});
	}

	cpu_vec
}
