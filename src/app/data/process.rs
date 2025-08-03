use std::{collections::BTreeMap, vec::Vec};

use hashbrown::HashMap;

use crate::collection::processes::{Pid, ProcessHarvest};

#[derive(Clone, Debug, Default)]
pub struct ProcessData {
    /// A PID to process data map.
    pub process_harvest: BTreeMap<Pid, ProcessHarvest>,

    /// A mapping between a process PID to any children process PIDs.
    pub process_parent_mapping: HashMap<Pid, Vec<Pid>>,

    /// PIDs corresponding to processes that have no parents.
    pub orphan_pids: Vec<Pid>,
}

impl ProcessData {
    pub(super) fn ingest(&mut self, list_of_processes: Vec<ProcessHarvest>) {
        self.process_parent_mapping.clear();

        // Reverse as otherwise the pid mappings are in the wrong order.
        list_of_processes.iter().rev().for_each(|process_harvest| {
            if let Some(parent_pid) = process_harvest.parent_pid {
                if let Some(entry) = self.process_parent_mapping.get_mut(&parent_pid) {
                    entry.push(process_harvest.pid);
                } else {
                    self.process_parent_mapping
                        .insert(parent_pid, vec![process_harvest.pid]);
                }
            }
        });

        self.process_parent_mapping.shrink_to_fit();

        let process_pid_map = list_of_processes
            .into_iter()
            .map(|process| (process.pid, process))
            .collect();
        self.process_harvest = process_pid_map;

        // We collect all processes that either:
        // - Do not have a parent PID (that is, they are orphan processes)
        // - Have a parent PID but we don't have the parent (we promote them as orphans)
        self.orphan_pids = self
            .process_harvest
            .iter()
            .filter_map(|(pid, process_harvest)| match process_harvest.parent_pid {
                Some(parent_pid) if self.process_harvest.contains_key(&parent_pid) => None,
                _ => Some(*pid),
            })
            .collect();
    }
}
