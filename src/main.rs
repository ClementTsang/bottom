use std::collections::BTreeMap;
use sysinfo::{ProcessExt, ProcessStatus, RefreshKind, System, SystemExt};

// TODO: Fix this - CPU Up, and CPU Down!
enum ProcessSorting {
	CPU,
	MEM,
	PID,
	Status,
}

fn main() {
	let mut system = System::new();
	system.refresh_all();
	draw_sorted_processes(ProcessSorting::CPU, true, &system);
}

// Possible process info struct?
#[derive(Debug)]
struct ProcessInfo<'a> {
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

fn draw_sorted_processes(sorting_method: ProcessSorting, reverse_order: bool, sys: &System) {
	let process_hashmap = sys.get_process_list();

	// Read into a btreemap, based on sorting type.
	// TODO: Evaluate whether this is too slow!

	let mut process_vector: Vec<sysinfo::Process> = process_hashmap.iter().map(|(_, process)| process.clone()).collect();

	match sorting_method {
			ProcessSorting::CPU => process_vector.sort_by(|a, b| {
				let a_usage = a.cpu_usage();
				let b_usage = b.cpu_usage();

				if a_usage > b_usage {
					if reverse_order {
						std::cmp::Ordering::Less
					} else {
						std::cmp::Ordering::Greater
					}
				} else if a_usage < b_usage {
					if reverse_order {
						std::cmp::Ordering::Greater
					} else {
						std::cmp::Ordering::Less
					}
				} else {
					std::cmp::Ordering::Equal
				}
			}),
			ProcessSorting::MEM => {}
			ProcessSorting::PID => {}
			ProcessSorting::Status => {}
		}

	let mut formatted_vector : Vec<ProcessInfo> = Vec::new();
	for process in &mut process_vector {
		formatted_vector.push(ProcessInfo {
			cpu_usage: process.cpu_usage(),
			command: process.name(),
			mem_usage: process.memory(),
			uptime: process.start_time(),
			pid: process.pid() as u32,
		});
	}

	for process in formatted_vector {
		println!("{:?}", process);
	}
}

fn get_timestamped_temperature() {}

fn draw_temperatures() {}

fn get_timestamped_cpu_data() {}

fn draw_cpu_data() {}

fn get_timestamped_ram_data() {}

fn draw_ram_data() {}

fn get_timestamped_network_data() {}

fn draw_network_data() {}

fn get_timestamped_drive_data() {}

fn draw_drive_usage_data() {}
