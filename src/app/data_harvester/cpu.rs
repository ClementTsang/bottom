use sysinfo::{ProcessorExt, System, SystemExt};

#[derive(Default, Debug, Clone)]
pub struct CpuData {
    pub cpu_prefix: String,
    pub cpu_count: Option<usize>,
    pub cpu_usage: f64,
}

pub type CpuHarvest = Vec<CpuData>;

pub fn get_cpu_data_list(sys: &mut System, show_average_cpu: bool) -> CpuHarvest {
    sys.refresh_cpu();

    let cpu_data = sys.get_processors();
    let avg_cpu_usage = sys.get_global_processor_info().get_cpu_usage();
    let mut cpu_vec = vec![];

    if show_average_cpu {
        cpu_vec.push(CpuData {
            cpu_prefix: "AVG".to_string(),
            cpu_count: None,
            cpu_usage: avg_cpu_usage as f64,
        });
    }

    for (itx, cpu) in cpu_data.iter().enumerate() {
        cpu_vec.push(CpuData {
            cpu_prefix: "CPU".to_string(),
            cpu_count: Some(itx),
            cpu_usage: f64::from(cpu.get_cpu_usage()),
        });
    }

    cpu_vec
}
