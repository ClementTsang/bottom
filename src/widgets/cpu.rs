struct TimedCPUData<'a> {
	cpu_name: &'a str,
	cpu_usage: f32,
	time: std::time::Duration,
}

fn get_timestamped_cpu_data() {}

pub fn get_cpu_data_list() {}
