use sysinfo::{ProcessorExt, System, SystemExt};

pub struct TimedCPUData {
	pub cpu_name : Box<str>,
	pub cpu_usage : u32,
	pub time : std::time::SystemTime,
}

pub fn get_cpu_data_list(sys : &System) -> Vec<TimedCPUData> {
	let cpu_data = sys.get_processor_list();
	let mut cpu_vec = Vec::new();

	for cpu in cpu_data {
		cpu_vec.push(TimedCPUData {
			cpu_name : Box::from(cpu.get_name()),
			cpu_usage : (cpu.get_cpu_usage() * 100_f32).ceil() as u32,
			time : std::time::SystemTime::now(),
		})
	}

	cpu_vec
}
