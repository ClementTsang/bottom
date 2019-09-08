use sysinfo::{ProcessorExt, System, SystemExt};

#[derive(Clone, Default)]
pub struct CPUData {
	pub cpu_name : Box<str>,
	pub cpu_usage : u32,
}

pub fn get_cpu_data_list(sys : &System) -> Result<Vec<CPUData>, heim::Error> {
	let cpu_data = sys.get_processor_list();
	let mut cpu_vec = Vec::new();

	for cpu in cpu_data {
		cpu_vec.push(CPUData {
			cpu_name : Box::from(cpu.get_name()),
			cpu_usage : (cpu.get_cpu_usage() * 100_f32).ceil() as u32,
		})
	}

	Ok(cpu_vec)
}
