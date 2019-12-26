use std::cmp::Ordering;
use std::{collections::HashMap, process::Command, time::Instant};
use sysinfo::{ProcessExt, System, SystemExt};

#[derive(Clone)]
pub enum ProcessSorting {
	CPU,
	MEM,
	PID,
	NAME,
}

impl Default for ProcessSorting {
	fn default() -> Self {
		ProcessSorting::CPU
	}
}

// Possible process info struct?
#[derive(Clone, Default)]
pub struct ProcessData {
	pub pid: u32,
	pub cpu_usage_percent: f64,
	pub mem_usage_percent: Option<f64>,
	pub mem_usage_kb: Option<u64>,
	pub command: String,
}

fn cpu_usage_calculation(prev_idle: &mut f64, prev_non_idle: &mut f64) -> std::io::Result<(f64, f64)> {
	// From SO answer: https://stackoverflow.com/a/23376195
	let mut path = std::path::PathBuf::new();
	path.push("/proc");
	path.push("stat");

	let stat_results = std::fs::read_to_string(path)?;
	let first_line = stat_results.split('\n').collect::<Vec<&str>>()[0];

	// TODO: Consider grabbing by number of threads instead, and summing the total?
	// ie: 4 threads, so: (prev - cur) / cpu_0 + ... + (prev - cur) / cpu_n instead?  This might be how top does it?
	let val = first_line.split_whitespace().collect::<Vec<&str>>();

	// SC in case that the parsing will fail due to length:
	if val.len() <= 10 {
		return Ok((1.0, 0.0)); // TODO: This is not the greatest...
	}

	let user: f64 = val[1].parse::<_>().unwrap_or(0_f64);
	let nice: f64 = val[2].parse::<_>().unwrap_or(0_f64);
	let system: f64 = val[3].parse::<_>().unwrap_or(0_f64);
	let idle: f64 = val[4].parse::<_>().unwrap_or(0_f64);
	let iowait: f64 = val[5].parse::<_>().unwrap_or(0_f64);
	let irq: f64 = val[6].parse::<_>().unwrap_or(0_f64);
	let softirq: f64 = val[7].parse::<_>().unwrap_or(0_f64);
	let steal: f64 = val[8].parse::<_>().unwrap_or(0_f64);
	let guest: f64 = val[9].parse::<_>().unwrap_or(0_f64);

	let idle = idle + iowait;
	let non_idle = user + nice + system + irq + softirq + steal + guest;

	let total = idle + non_idle;
	let prev_total = *prev_idle + *prev_non_idle;

	let total_delta: f64 = total - prev_total;
	let idle_delta: f64 = idle - *prev_idle;

	//debug!("Vangelis function: CPU PERCENT: {}", (total_delta - idle_delta) / total_delta * 100_f64);

	*prev_idle = idle;
	*prev_non_idle = non_idle;

	let result = if total_delta - idle_delta != 0_f64 {
		total_delta - idle_delta
	} else {
		1_f64
	};

	let cpu_percentage = if total_delta != 0_f64 { result / total_delta } else { 0_f64 };

	Ok((result, cpu_percentage)) // This works, REALLY damn well.  The percentage check is within like 2% of the sysinfo one.
}

fn get_ordering<T: std::cmp::PartialOrd>(a_val: T, b_val: T, reverse_order: bool) -> std::cmp::Ordering {
	match a_val.partial_cmp(&b_val) {
		Some(x) => match x {
			Ordering::Greater => {
				if reverse_order {
					std::cmp::Ordering::Less
				} else {
					std::cmp::Ordering::Greater
				}
			}
			Ordering::Less => {
				if reverse_order {
					std::cmp::Ordering::Greater
				} else {
					std::cmp::Ordering::Less
				}
			}
			Ordering::Equal => Ordering::Equal,
		},
		None => Ordering::Equal, // I don't really like this but I think it's fine...
	}
}

fn get_process_cpu_stats(pid: u32) -> std::io::Result<f64> {
	let mut path = std::path::PathBuf::new();
	path.push("/proc");
	path.push(&pid.to_string());
	path.push("stat");

	let stat_results = std::fs::read_to_string(path)?;
	let val = stat_results.split_whitespace().collect::<Vec<&str>>();
	let utime = val[13].parse::<f64>().unwrap_or(0_f64);
	let stime = val[14].parse::<f64>().unwrap_or(0_f64);

	//debug!("PID: {}, utime: {}, stime: {}", pid, utime, stime);

	Ok(utime + stime) // This seems to match top...
}

/// Note that cpu_percentage should be represented WITHOUT the \times 100 factor!
fn linux_cpu_usage(pid: u32, cpu_usage: f64, cpu_percentage: f64, previous_pid_stats: &mut HashMap<String, (f64, Instant)>) -> std::io::Result<f64> {
	// Based heavily on https://stackoverflow.com/a/23376195 and https://stackoverflow.com/a/1424556
	let before_proc_val: f64 = if previous_pid_stats.contains_key(&pid.to_string()) {
		previous_pid_stats.get(&pid.to_string()).unwrap_or(&(0_f64, Instant::now())).0
	} else {
		0_f64
	};
	let after_proc_val = get_process_cpu_stats(pid)?;

	/*debug!(
		"PID - {} - Before: {}, After: {}, CPU: {}, Percentage: {}",
		pid,
		before_proc_val,
		after_proc_val,
		cpu_usage,
		(after_proc_val - before_proc_val) / cpu_usage * 100_f64
	);*/

	let entry = previous_pid_stats.entry(pid.to_string()).or_insert((after_proc_val, Instant::now()));
	*entry = (after_proc_val, Instant::now());
	Ok((after_proc_val - before_proc_val) / cpu_usage * 100_f64 * cpu_percentage)
}

fn convert_ps(
	process: &str, cpu_usage: f64, cpu_percentage: f64, prev_pid_stats: &mut HashMap<String, (f64, Instant)>,
) -> std::io::Result<ProcessData> {
	if process.trim().to_string().is_empty() {
		return Ok(ProcessData {
			pid: 0,
			command: "".to_string(),
			mem_usage_percent: None,
			mem_usage_kb: None,
			cpu_usage_percent: 0_f64,
		});
	}

	let pid = (&process[..11]).trim().to_string().parse::<u32>().unwrap_or(0);
	let command = (&process[11..61]).trim().to_string();
	let mem_usage_percent = Some((&process[62..]).trim().to_string().parse::<f64>().unwrap_or(0_f64));

	Ok(ProcessData {
		pid,
		command,
		mem_usage_percent,
		mem_usage_kb: None,
		cpu_usage_percent: linux_cpu_usage(pid, cpu_usage, cpu_percentage, prev_pid_stats)?,
	})
}

pub async fn get_sorted_processes_list(
	sys: &System, prev_idle: &mut f64, prev_non_idle: &mut f64, prev_pid_stats: &mut std::collections::HashMap<String, (f64, Instant)>,
) -> crate::utils::error::Result<Vec<ProcessData>> {
	let mut process_vector: Vec<ProcessData> = Vec::new();

	if cfg!(target_os = "linux") {
		// Linux specific - this is a massive pain... ugh.

		let ps_result = Command::new("ps").args(&["-axo", "pid:10,comm:50,%mem:5", "--noheader"]).output()?;
		let ps_stdout = String::from_utf8_lossy(&ps_result.stdout);
		let split_string = ps_stdout.split('\n');
		if let Ok((cpu_usage, cpu_percentage)) = cpu_usage_calculation(prev_idle, prev_non_idle) {
			let process_stream = split_string.collect::<Vec<&str>>();

			for process in process_stream {
				if let Ok(process_object) = convert_ps(process, cpu_usage, cpu_percentage, prev_pid_stats) {
					if !process_object.command.is_empty() {
						process_vector.push(process_object);
					}
				}
			}
		}
	} else {
		// Windows et al.

		let process_hashmap = sys.get_process_list();
		for process_val in process_hashmap.values() {
			process_vector.push(ProcessData {
				pid: process_val.pid() as u32,
				command: process_val.name().to_string(),
				mem_usage_percent: None,
				mem_usage_kb: Some(process_val.memory()),
				cpu_usage_percent: f64::from(process_val.cpu_usage()),
			});
		}
	}

	Ok(process_vector)
}

pub fn sort_processes(process_vector: &mut Vec<ProcessData>, sorting_method: &ProcessSorting, reverse_order: bool) {
	match sorting_method {
		// Always sort alphabetically first!
		ProcessSorting::CPU => {
			process_vector.sort_by(|a, b| get_ordering(&a.command, &b.command, false));
			process_vector.sort_by(|a, b| get_ordering(a.cpu_usage_percent, b.cpu_usage_percent, reverse_order));
		}
		ProcessSorting::MEM => {
			process_vector.sort_by(|a, b| get_ordering(&a.command, &b.command, false));
			process_vector.sort_by(|a, b| get_ordering(a.mem_usage_percent, b.mem_usage_percent, reverse_order));
		}
		ProcessSorting::PID => {
			process_vector.sort_by(|a, b| get_ordering(&a.command, &b.command, false));
			process_vector.sort_by(|a, b| get_ordering(a.pid, b.pid, reverse_order));
		}
		ProcessSorting::NAME => process_vector.sort_by(|a, b| get_ordering(&a.command, &b.command, reverse_order)),
	}
}
