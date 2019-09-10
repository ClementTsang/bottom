use heim_common::{
	prelude::{StreamExt, TryStreamExt},
	units,
};
use std::process::Command;

#[allow(dead_code)]
#[derive(Clone)]
pub enum ProcessSorting {
	CPU,
	MEM,
	PID,
	NAME,
}

// Possible process info struct?
#[derive(Clone, Default)]
pub struct ProcessData {
	pub pid : u32,
	pub cpu_usage_percent : f64,
	pub mem_usage_percent : f64,
	pub command : String,
}

fn get_ordering<T : std::cmp::PartialOrd>(a_val : T, b_val : T, reverse_order : bool) -> std::cmp::Ordering {
	if a_val > b_val {
		if reverse_order {
			std::cmp::Ordering::Less
		}
		else {
			std::cmp::Ordering::Greater
		}
	}
	else if a_val < b_val {
		if reverse_order {
			std::cmp::Ordering::Greater
		}
		else {
			std::cmp::Ordering::Less
		}
	}
	else {
		std::cmp::Ordering::Equal
	}
}

async fn non_linux_cpu_usage(process : heim::process::Process) -> heim::process::ProcessResult<(heim::process::Process, heim_common::units::Ratio)> {
	let usage_1 = process.cpu_usage().await?;
	futures_timer::Delay::new(std::time::Duration::from_millis(100)).await?;
	let usage_2 = process.cpu_usage().await?;

	Ok((process, usage_2 - usage_1))
}

fn get_process_cpu_stats(pid : u32) -> std::io::Result<f64> {
	let mut path = std::path::PathBuf::new();
	path.push("/proc");
	path.push(&pid.to_string());
	path.push("stat");

	let stat_results = std::fs::read_to_string(path)?;
	let val = stat_results.split_whitespace().collect::<Vec<&str>>();
	Ok(val[13].parse::<f64>().unwrap_or(0_f64) + val[14].parse::<f64>().unwrap_or(0_f64))
}

fn get_cpu_use_val() -> std::io::Result<f64> {
	let mut path = std::path::PathBuf::new();
	path.push("/proc");
	path.push("stat");

	let stat_results = std::fs::read_to_string(path)?;
	let first_line = stat_results.split('\n').collect::<Vec<&str>>()[0];
	let val = first_line.split_whitespace().collect::<Vec<&str>>();
	Ok(val[0].parse::<f64>().unwrap_or(0_f64) + val[1].parse::<f64>().unwrap_or(0_f64) + val[2].parse::<f64>().unwrap_or(0_f64) + val[3].parse::<f64>().unwrap_or(0_f64))
}

async fn linux_cpu_usage(pid : u32) -> std::io::Result<f64> {
	// Based heavily on https://stackoverflow.com/a/23376195 and https://stackoverflow.com/a/1424556
	let before_proc_val = get_process_cpu_stats(pid)?;
	let before_cpu_val = get_cpu_use_val()?;
	futures_timer::Delay::new(std::time::Duration::from_millis(1000)).await.unwrap();
	let after_proc_val = get_process_cpu_stats(pid)?;
	let after_cpu_val = get_cpu_use_val()?;

	Ok((after_proc_val - before_proc_val) / (after_cpu_val - before_cpu_val) * 100_f64)
}

async fn convert_ps(process : &str) -> std::io::Result<ProcessData> {
	let mut result = process.split_whitespace();
	let pid = result.next().unwrap_or("").parse::<u32>().unwrap_or(0);
	Ok(ProcessData {
		pid,
		command : result.next().unwrap_or("").to_string(),
		mem_usage_percent : result.next().unwrap_or("").parse::<f64>().unwrap_or(0_f64),
		cpu_usage_percent : linux_cpu_usage(pid).await?,
	})
}

pub async fn get_sorted_processes_list(total_mem : u64) -> Result<Vec<ProcessData>, heim::Error> {
	let mut process_vector : Vec<ProcessData> = Vec::new();

	if cfg!(target_os = "linux") {
		// Linux specific - this is a massive pain... ugh.
		let ps_result = Command::new("ps").args(&["-axo", "pid,comm,%mem", "--noheader"]).output().expect("Failed to execute.");
		let ps_stdout = String::from_utf8_lossy(&ps_result.stdout);
		let split_string = ps_stdout.split('\n');
		let mut process_stream = futures::stream::iter::<_>(split_string.collect::<Vec<&str>>()).map(convert_ps).buffer_unordered(std::usize::MAX);

		while let Some(process) = process_stream.next().await {
			if let Ok(process) = process {
				process_vector.push(process);
			}
		}
	}
	else if cfg!(target_os = "windows") {
		// Windows
		let mut process_stream = heim::process::processes().map_ok(non_linux_cpu_usage).try_buffer_unordered(std::usize::MAX);

		let mut process_vector : Vec<ProcessData> = Vec::new();
		while let Some(process) = process_stream.next().await {
			if let Ok(process) = process {
				let (process, cpu_usage) = process;
				let mem_measurement = process.memory().await;
				if let Ok(mem_measurement) = mem_measurement {
					process_vector.push(ProcessData {
						command : process.name().await.unwrap_or_else(|_| "".to_string()),
						pid : process.pid() as u32,
						cpu_usage_percent : f64::from(cpu_usage.get::<units::ratio::percent>()),
						mem_usage_percent : mem_measurement.rss().get::<units::information::megabyte>() as f64 / total_mem as f64 * 100_f64,
					});
				}
			}
		}
	}
	else if cfg!(target_os = "macos") {
		// macOS
		dbg!("Mac"); // TODO: Remove
	}
	else {
		dbg!("Else"); // TODO: Remove
	}

	Ok(process_vector)
}

pub fn sort_processes(process_vector : &mut Vec<ProcessData>, sorting_method : &ProcessSorting, reverse_order : bool) {
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
