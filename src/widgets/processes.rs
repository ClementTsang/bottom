use heim_common::prelude::StreamExt;

pub enum ProcessSorting {
	CPU,
	MEM,
	PID,
	NAME,
}

// Possible process info struct?
#[derive(Debug)]
pub struct ProcessInfo {
	pid : u32,
	cpu_usage : f32,
	mem_usage : u64,
	uptime : u64,
	command : Box<str>,
	// TODO: Env?
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

pub async fn get_sorted_processes_list(sorting_method : ProcessSorting, reverse_order : bool) -> Vec<ProcessInfo> {
	let mut process_stream = heim::process::processes();

	// TODO: Evaluate whether this is too slow!
	// TODO: Should I filter out blank command names?

	let mut process_vector : Vec<ProcessInfo> = Vec::new();
	while let Some(process) = process_stream.next().await {}

	match sorting_method {
		ProcessSorting::CPU => process_vector.sort_by(|a, b| get_ordering(1, 2, reverse_order)),
		ProcessSorting::MEM => process_vector.sort_by(|a, b| get_ordering(1, 2, reverse_order)),
		ProcessSorting::PID => process_vector.sort_by(|a, b| get_ordering(1, 2, reverse_order)),
		ProcessSorting::NAME => process_vector.sort_by(|a, b| get_ordering(1, 2, reverse_order)),
	}

	process_vector
}
