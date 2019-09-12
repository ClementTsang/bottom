use std::time::Instant;
use sysinfo::{ProcessorExt, System, SystemExt};

#[derive(Clone)]
pub struct CPUData {
	pub cpu_name : Box<str>,
	pub cpu_usage : f64,
}

#[derive(Clone)]
pub struct CPUPackage {
	pub cpu_vec : Vec<CPUData>,
	pub instant : Instant,
}

pub fn get_cpu_data_list(sys : &System) -> Result<CPUPackage, heim::Error> {
	let cpu_data = sys.get_processor_list();
	let mut cpu_vec = Vec::new();

	for cpu in cpu_data {
		cpu_vec.push(CPUData {
			cpu_name : Box::from(cpu.get_name()),
			cpu_usage : f64::from(cpu.get_cpu_usage()) * 100_f64,
		})
	}

	Ok(CPUPackage { cpu_vec, instant : Instant::now() })
}
