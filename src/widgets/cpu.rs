use std::time::Instant;
use sysinfo::{ProcessorExt, System, SystemExt};

#[derive(Clone)]
pub struct CPUData {
	pub cpu_name : Box<str>,
	pub cpu_usage : u32,
	pub instant : Instant,
}

pub fn get_cpu_data_list(sys : &System) -> Result<Vec<CPUData>, heim::Error> {
	let cpu_data = sys.get_processor_list();
	let mut cpu_vec = Vec::new();

	for cpu in cpu_data {
		cpu_vec.push(CPUData {
			cpu_name : Box::from(cpu.get_name()),
			cpu_usage : (cpu.get_cpu_usage() * 100_f32).ceil() as u32,
			instant : Instant::now(),
		})
	}

	Ok(cpu_vec)
}
