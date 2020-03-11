use sysinfo::{ProcessorExt, System, SystemExt};

#[derive(Default, Debug, Clone)]
pub struct CPUData {
    pub cpu_name: String,
    pub cpu_usage: f64,
}

pub type CPUHarvest = Vec<CPUData>;

pub fn get_cpu_data_list(sys: &System, show_average_cpu: bool) -> CPUHarvest {
    let cpu_data = sys.get_processors();
    let avg_cpu_usage = sys.get_global_processor_info().get_cpu_usage();
    let mut cpu_vec = vec![];

    if show_average_cpu {
        cpu_vec.push(CPUData {
            cpu_name: "AVG".to_string(),
            cpu_usage: avg_cpu_usage as f64,
        });
    }

    for cpu in cpu_data {
        cpu_vec.push(CPUData {
            cpu_name: cpu.get_name().to_uppercase(),
            cpu_usage: f64::from(cpu.get_cpu_usage()),
        });
    }

    cpu_vec
}
