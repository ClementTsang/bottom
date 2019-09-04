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
	draw_sorted_processes(ProcessSorting::CPU, &system);
}

#[derive(Debug)]
struct ProcessInfo<'a> {
	pid: u32,
	cpu_usage: f32,
	mem_usage: u64,
	uptime: u64,
	command: &'a str,
	status: &'a str,
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

fn draw_sorted_processes(sorting_method: ProcessSorting, sys: &System) {
	let process_hashmap = sys.get_process_list();

	// Read into a btreemap, based on sorting type.
	// TODO: Evaluate whether this is too slow!

	let mut process_tree: BTreeMap<String, ProcessInfo> = BTreeMap::new();

	for (pid, process) in process_hashmap {
		let new_process_info = ProcessInfo {
			pid: *pid as u32,
			cpu_usage: process.cpu_usage(),
			mem_usage: process.memory(),
			uptime: process.start_time(), // TODO: This is not correct rn
			command: process.name(),
			status: get_status(process.status()),
		};
		match sorting_method {
			ProcessSorting::CPU => {
				process_tree.insert(process.cpu_usage().to_string(), new_process_info);
			}
			ProcessSorting::MEM => {
				process_tree.insert(process.memory().to_string(), new_process_info);
			}
			ProcessSorting::PID => {
				process_tree.insert(pid.to_string(), new_process_info);
			}
			ProcessSorting::Status => {
				process_tree.insert(get_status(process.status()).to_string(), new_process_info);
			}
		}
	}

	for (k, v) in process_tree {
		println!("Key: {}, Val: {:?}", k, v);
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
