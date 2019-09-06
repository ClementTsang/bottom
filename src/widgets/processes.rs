use sysinfo::{ProcessExt, ProcessStatus, System, SystemExt};

// TODO: Fix this - CPU Up, and CPU Down!
pub enum ProcessSorting {
	CPU,
	MEM,
	PID,
	NAME,
}

// Possible process info struct?
#[derive(Debug)]
pub struct ProcessInfo<'a> {
	pid: u32,
	cpu_usage: f32,
	mem_usage: u64,
	uptime: u64,
	command: &'a str,
	//status: &'a str,
	// TODO: Env?
}

fn get_status(status: ProcessStatus) -> &'static str {
	match status {
		ProcessStatus::Idle => "Idle",
		ProcessStatus::Run => "Run",
		ProcessStatus::Sleep => "Sleep",
		ProcessStatus::Zombie => "Zombie",
		ProcessStatus::Tracing => "Tracing",
		ProcessStatus::Dead => "Dead",
		_ => "Unknown",
	}
}

fn get_ordering<T: std::cmp::PartialOrd>(a_val: T, b_val: T, reverse_order: bool) -> std::cmp::Ordering {
	if a_val > b_val {
		if reverse_order {
			std::cmp::Ordering::Less
		} else {
			std::cmp::Ordering::Greater
		}
	} else if a_val < b_val {
		if reverse_order {
			std::cmp::Ordering::Greater
		} else {
			std::cmp::Ordering::Less
		}
	} else {
		std::cmp::Ordering::Equal
	}
}

pub fn get_sorted_processes_list(sorting_method: ProcessSorting, reverse_order: bool, sys: &System) {
	let process_hashmap = sys.get_process_list();

	// TODO: Evaluate whether this is too slow!
	// TODO: Should I filter out blank command names?
	let mut process_vector: Vec<sysinfo::Process> = process_hashmap.iter().map(|(_, process)| process.clone()).collect();

	match sorting_method {
		ProcessSorting::CPU => process_vector.sort_by(|a, b| get_ordering(a.cpu_usage(), b.cpu_usage(), reverse_order)),
		ProcessSorting::MEM => process_vector.sort_by(|a, b| get_ordering(a.memory(), b.memory(), reverse_order)),
		ProcessSorting::PID => process_vector.sort_by(|a, b| get_ordering(a.pid(), b.pid(), reverse_order)),
		ProcessSorting::NAME => process_vector.sort_by(|a, b| get_ordering(a.name(), b.name(), reverse_order)),
	}

	let mut formatted_vector: Vec<ProcessInfo> = Vec::new();
	for process in &mut process_vector {
		formatted_vector.push(ProcessInfo {
			cpu_usage: process.cpu_usage(),
			command: process.name(),
			mem_usage: process.memory(),
			uptime: process.start_time(),
			pid: process.pid() as u32,
		});
	}

	// TODO: For debugging, remove.
	for process in formatted_vector {
		println!("{:?}", process);
	}
}
