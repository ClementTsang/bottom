use heim_common::{
	prelude::{StreamExt, TryStreamExt},
	units,
};

#[allow(dead_code)]
pub enum ProcessSorting {
	CPU,
	MEM,
	PID,
	NAME,
}

// Possible process info struct?
#[derive(Clone, Default)]
pub struct ProcessInfo {
	pub pid : u32,
	pub cpu_usage_percent : f32,
	pub mem_usage_in_mb : u64,
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

async fn cpu_usage(process : heim::process::Process) -> heim::process::ProcessResult<(heim::process::Process, heim_common::units::Ratio)> {
	let usage_1 = process.cpu_usage().await?;
	futures_timer::Delay::new(std::time::Duration::from_millis(100)).await?;
	let usage_2 = process.cpu_usage().await?;

	Ok((process, usage_2 - usage_1))
}

pub async fn get_sorted_processes_list(sorting_method : ProcessSorting, reverse_order : bool) -> Result<Vec<ProcessInfo>, heim::Error> {
	let mut process_stream = heim::process::processes().map_ok(cpu_usage).try_buffer_unordered(std::usize::MAX);

	let mut process_vector : Vec<ProcessInfo> = Vec::new();
	while let Some(process) = process_stream.next().await {
		if let Ok(process) = process {
			let (process, cpu_usage) = process;
			let mem_measurement = process.memory().await;
			if let Ok(mem_measurement) = mem_measurement {
				/*
				// Unsure whether I want to implement this by grouping together process names...?
				let mut process_info = process_hashmap.entry(command_name.to_string()).or_insert(ProcessInfo {
					command : command_name,
					pid : process.pid() as u32,
					cpu_usage_percent : cpu_usage.get::<units::ratio::percent>(),
					mem_usage_in_mb : mem_measurement.rss().get::<units::information::megabyte>(),
				});
				*/

				process_vector.push(ProcessInfo {
					command : process.name().await.unwrap_or_else(|_| "".to_string()),
					pid : process.pid() as u32,
					cpu_usage_percent : cpu_usage.get::<units::ratio::percent>(),
					mem_usage_in_mb : mem_measurement.rss().get::<units::information::megabyte>(),
				});
			}
		}
	}
	sort_processes(sorting_method, &mut process_vector, reverse_order);

	Ok(process_vector)
}

pub fn sort_processes(sorting_method : ProcessSorting, process_vector : &mut Vec<ProcessInfo>, reverse_order : bool) {
	match sorting_method {
		ProcessSorting::CPU => process_vector.sort_by(|a, b| get_ordering(a.cpu_usage_percent, b.cpu_usage_percent, reverse_order)),
		ProcessSorting::MEM => process_vector.sort_by(|a, b| get_ordering(a.mem_usage_in_mb, b.mem_usage_in_mb, reverse_order)),
		ProcessSorting::PID => process_vector.sort_by(|a, b| get_ordering(a.pid, b.pid, reverse_order)),
		ProcessSorting::NAME => process_vector.sort_by(|a, b| get_ordering(&a.command, &b.command, reverse_order)),
	}
}
