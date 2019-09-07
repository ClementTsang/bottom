use sysinfo::{ProcessorExt, System, SystemExt};

pub struct CPUData {
	pub cpu_name : Box<str>,
	pub cpu_usage : u32,
}

pub struct TimedCPUPackages {
	pub processor_list : Vec<CPUData>,
	pub time : std::time::SystemTime,
}

pub fn get_cpu_data_list(sys : &System) -> TimedCPUPackages {
	let cpu_data = sys.get_processor_list();
	let mut cpu_vec = Vec::new();

	for cpu in cpu_data {
		cpu_vec.push(CPUData {
			cpu_name : Box::from(cpu.get_name()),
			cpu_usage : (cpu.get_cpu_usage() * 100_f32).ceil() as u32,
		})
	}

	TimedCPUPackages {
		processor_list : cpu_vec,
		time : std::time::SystemTime::now(),
	}
}

pub fn is_cpu_data_old() -> bool {
	true
}
