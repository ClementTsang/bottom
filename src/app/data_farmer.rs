use itertools::Itertools;
/// In charge of cleaning, processing, and managing data.  I couldn't think of
/// a better name for the file.  Since I called data collection "harvesting",
/// then this is the farmer I guess.
///
/// Essentially the main goal is to shift the initial calculation and distribution
/// of joiner points and data to one central location that will only do it
/// *once* upon receiving the data --- as opposed to doing it on canvas draw,
/// which will be a costly process.
///
/// This will also handle the *cleaning* of stale data.  That should be done
/// in some manner (timer on another thread, some loop) that will occasionally
/// call the purging function.  Failure to do so *will* result in a growing
/// memory usage and higher CPU usage - you will be trying to process more and
/// more points as this is used!
use once_cell::sync::Lazy;
use rustc_hash::FxHashMap;

use std::{collections::HashMap, time::Instant, vec::Vec};

#[cfg(feature = "battery")]
use crate::data_harvester::batteries;

use crate::{
    data_harvester::{cpu, disks, memory, network, processes, temperature, Data},
    utils::gen_util::{get_decimal_bytes, GIGA_LIMIT},
    Pid,
};
use regex::Regex;

use super::data_harvester::processes::ProcessHarvest;

#[derive(Clone, Debug, Default)]
pub struct TimedData {
    pub rx_data: f64,
    pub tx_data: f64,
    pub cpu_data: Vec<f64>,
    pub load_avg_data: [f32; 3],
    pub mem_data: Option<f64>,
    pub swap_data: Option<f64>,
}

#[derive(Clone, Debug, Default)]
pub struct ProcessData {
    /// A PID to process data map.
    pub process_harvest: FxHashMap<Pid, ProcessHarvest>,

    /// A mapping from a process name to any PID with that name.
    pub process_name_pid_map: HashMap<String, Vec<Pid>>,

    /// A mapping from a process command to any PID with that name.
    pub process_cmd_pid_map: HashMap<String, Vec<Pid>>,

    /// A mapping between a process PID to any children process PIDs.
    pub process_parent_mapping: FxHashMap<Pid, Vec<Pid>>,

    /// PIDs corresponding to processes that have no parents.
    pub orphan_pids: Vec<Pid>,
}

impl ProcessData {
    fn ingest(&mut self, list_of_processes: Vec<ProcessHarvest>) {
        // TODO: [Optimization] Probably more efficient to all of this in the data collection step, but it's fine for now.
        self.process_name_pid_map.clear();
        self.process_cmd_pid_map.clear();
        self.process_parent_mapping.clear();

        // Reverse as otherwise the pid mappings are in the wrong order.
        list_of_processes.iter().rev().for_each(|process_harvest| {
            if let Some(entry) = self.process_name_pid_map.get_mut(&process_harvest.name) {
                entry.push(process_harvest.pid);
            } else {
                self.process_name_pid_map
                    .insert(process_harvest.name.to_string(), vec![process_harvest.pid]);
            }

            if let Some(entry) = self.process_cmd_pid_map.get_mut(&process_harvest.command) {
                entry.push(process_harvest.pid);
            } else {
                self.process_cmd_pid_map.insert(
                    process_harvest.command.to_string(),
                    vec![process_harvest.pid],
                );
            }

            if let Some(parent_pid) = process_harvest.parent_pid {
                if let Some(entry) = self.process_parent_mapping.get_mut(&parent_pid) {
                    entry.push(process_harvest.pid);
                } else {
                    self.process_parent_mapping
                        .insert(parent_pid, vec![process_harvest.pid]);
                }
            }
        });

        self.process_name_pid_map.shrink_to_fit();
        self.process_cmd_pid_map.shrink_to_fit();
        self.process_parent_mapping.shrink_to_fit();

        let process_pid_map = list_of_processes
            .into_iter()
            .map(|process| (process.pid, process))
            .collect();
        self.process_harvest = process_pid_map;

        // This also needs a quick sort + reverse to be in the correct order.
        self.orphan_pids = self
            .process_harvest
            .iter()
            .filter_map(|(pid, process_harvest)| {
                if let Some(parent_pid) = process_harvest.parent_pid {
                    if self.process_harvest.contains_key(&parent_pid) {
                        None
                    } else {
                        Some(*pid)
                    }
                } else {
                    Some(*pid)
                }
            })
            .sorted()
            .rev()
            .collect();
    }
}

/// AppCollection represents the pooled data stored within the main app
/// thread.  Basically stores a (occasionally cleaned) record of the data
/// collected, and what is needed to convert into a displayable form.
///
/// If the app is *frozen* - that is, we do not want to *display* any changing
/// data, keep updating this. As of 2021-09-08, we just clone the current collection
/// when it freezes to have a snapshot floating around.
///
/// Note that with this method, the *app* thread is responsible for cleaning -
/// not the data collector.
#[derive(Clone, Debug)]
pub struct DataCollection {
    pub current_instant: Instant, // TODO: [Refactor] Can I get rid of this? If I could, then I could just use #[derive(Default)] too!
    pub timed_data_vec: Vec<(Instant, TimedData)>,
    pub network_harvest: network::NetworkHarvest,
    pub memory_harvest: memory::MemHarvest,
    pub swap_harvest: memory::MemHarvest,
    pub cpu_harvest: cpu::CpuHarvest,
    pub load_avg_harvest: cpu::LoadAvgHarvest,
    pub process_data: ProcessData,
    pub disk_harvest: Vec<disks::DiskHarvest>,
    pub io_harvest: disks::IoHarvest,
    pub io_labels_and_prev: Vec<((u64, u64), (u64, u64))>,
    pub io_labels: Vec<(String, String)>,
    pub temp_harvest: Vec<temperature::TempHarvest>,
    #[cfg(feature = "battery")]
    pub battery_harvest: Vec<batteries::BatteryHarvest>,
}

impl Default for DataCollection {
    fn default() -> Self {
        DataCollection {
            current_instant: Instant::now(),
            timed_data_vec: Default::default(),
            network_harvest: Default::default(),
            memory_harvest: Default::default(),
            swap_harvest: Default::default(),
            cpu_harvest: Default::default(),
            load_avg_harvest: Default::default(),
            process_data: Default::default(),
            disk_harvest: Default::default(),
            io_harvest: Default::default(),
            io_labels_and_prev: Default::default(),
            io_labels: Default::default(),
            temp_harvest: Default::default(),
            #[cfg(feature = "battery")]
            battery_harvest: Default::default(),
        }
    }
}

impl DataCollection {
    pub fn reset(&mut self) {
        self.timed_data_vec = Default::default();
        self.network_harvest = Default::default();
        self.memory_harvest = Default::default();
        self.swap_harvest = Default::default();
        self.cpu_harvest = Default::default();
        self.process_data = Default::default();
        self.disk_harvest = Default::default();
        self.io_harvest = Default::default();
        self.io_labels_and_prev = Default::default();
        self.temp_harvest = Default::default();
        #[cfg(feature = "battery")]
        {
            self.battery_harvest = Vec::default();
        }
    }

    pub fn clean_data(&mut self, max_time_millis: u64) {
        let current_time = Instant::now();

        let remove_index = match self
            .timed_data_vec
            .binary_search_by(|(instant, _timed_data)| {
                current_time
                    .duration_since(*instant)
                    .as_millis()
                    .cmp(&(max_time_millis as u128))
                    .reverse()
            }) {
            Ok(index) => index,
            Err(index) => index,
        };

        self.timed_data_vec.drain(0..remove_index);
    }

    pub fn eat_data(&mut self, harvested_data: Box<Data>) {
        let harvested_time = harvested_data.last_collection_time;
        let mut new_entry = TimedData::default();

        // Network
        if let Some(network) = harvested_data.network {
            self.eat_network(network, &mut new_entry);
        }

        // Memory and Swap
        if let (Some(memory), Some(swap)) = (harvested_data.memory, harvested_data.swap) {
            self.eat_memory_and_swap(memory, swap, &mut new_entry);
        }

        // CPU
        if let Some(cpu) = harvested_data.cpu {
            self.eat_cpu(cpu, &mut new_entry);
        }

        // Load Average
        if let Some(load_avg) = harvested_data.load_avg {
            self.eat_load_avg(load_avg, &mut new_entry);
        }

        // Temp
        if let Some(temperature_sensors) = harvested_data.temperature_sensors {
            self.eat_temp(temperature_sensors);
        }

        // Disks
        if let Some(disks) = harvested_data.disks {
            if let Some(io) = harvested_data.io {
                self.eat_disks(disks, io, harvested_time);
            }
        }

        // Processes
        if let Some(list_of_processes) = harvested_data.list_of_processes {
            self.eat_proc(list_of_processes);
        }

        #[cfg(feature = "battery")]
        {
            // Battery
            if let Some(list_of_batteries) = harvested_data.list_of_batteries {
                self.eat_battery(list_of_batteries);
            }
        }

        // And we're done eating.  Update time and push the new entry!
        self.current_instant = harvested_time;
        self.timed_data_vec.push((harvested_time, new_entry));
    }

    fn eat_memory_and_swap(
        &mut self, memory: memory::MemHarvest, swap: memory::MemHarvest, new_entry: &mut TimedData,
    ) {
        // Memory
        new_entry.mem_data = memory.use_percent;

        // Swap
        new_entry.swap_data = swap.use_percent;

        // In addition copy over latest data for easy reference
        self.memory_harvest = memory;
        self.swap_harvest = swap;
    }

    fn eat_network(&mut self, network: network::NetworkHarvest, new_entry: &mut TimedData) {
        // RX
        if network.rx > 0 {
            new_entry.rx_data = network.rx as f64;
        }

        // TX
        if network.tx > 0 {
            new_entry.tx_data = network.tx as f64;
        }

        // In addition copy over latest data for easy reference
        self.network_harvest = network;
    }

    fn eat_cpu(&mut self, cpu: Vec<cpu::CpuData>, new_entry: &mut TimedData) {
        // Note this only pre-calculates the data points - the names will be
        // within the local copy of cpu_harvest.  Since it's all sequential
        // it probably doesn't matter anyways.
        cpu.iter()
            .for_each(|cpu| new_entry.cpu_data.push(cpu.cpu_usage));

        self.cpu_harvest = cpu.to_vec();
    }

    fn eat_load_avg(&mut self, load_avg: cpu::LoadAvgHarvest, new_entry: &mut TimedData) {
        new_entry.load_avg_data = load_avg;

        self.load_avg_harvest = load_avg;
    }

    fn eat_temp(&mut self, temperature_sensors: Vec<temperature::TempHarvest>) {
        self.temp_harvest = temperature_sensors.to_vec();
    }

    fn eat_disks(
        &mut self, disks: Vec<disks::DiskHarvest>, io: disks::IoHarvest, harvested_time: Instant,
    ) {
        let time_since_last_harvest = harvested_time
            .duration_since(self.current_instant)
            .as_secs_f64();

        for (itx, device) in disks.iter().enumerate() {
            if let Some(trim) = device.name.split('/').last() {
                let io_device = if cfg!(target_os = "macos") {
                    // Must trim one level further!
                    static DISK_REGEX: Lazy<Regex> = Lazy::new(|| Regex::new(r"disk\d+").unwrap());
                    if let Some(disk_trim) = DISK_REGEX.find(trim) {
                        io.get(disk_trim.as_str())
                    } else {
                        None
                    }
                } else {
                    io.get(trim)
                };

                if let Some(io_device) = io_device {
                    let (io_r_pt, io_w_pt) = if let Some(io) = io_device {
                        (io.read_bytes, io.write_bytes)
                    } else {
                        (0, 0)
                    };

                    if self.io_labels.len() <= itx {
                        self.io_labels.push((String::default(), String::default()));
                    }

                    if self.io_labels_and_prev.len() <= itx {
                        self.io_labels_and_prev.push(((0, 0), (io_r_pt, io_w_pt)));
                    }

                    if let Some((io_curr, io_prev)) = self.io_labels_and_prev.get_mut(itx) {
                        let r_rate = ((io_r_pt.saturating_sub(io_prev.0)) as f64
                            / time_since_last_harvest)
                            .round() as u64;
                        let w_rate = ((io_w_pt.saturating_sub(io_prev.1)) as f64
                            / time_since_last_harvest)
                            .round() as u64;

                        *io_curr = (r_rate, w_rate);
                        *io_prev = (io_r_pt, io_w_pt);

                        if let Some(io_labels) = self.io_labels.get_mut(itx) {
                            let converted_read = get_decimal_bytes(r_rate);
                            let converted_write = get_decimal_bytes(w_rate);
                            *io_labels = (
                                if r_rate >= GIGA_LIMIT {
                                    format!("{:.*}{}/s", 1, converted_read.0, converted_read.1)
                                } else {
                                    format!("{:.*}{}/s", 0, converted_read.0, converted_read.1)
                                },
                                if w_rate >= GIGA_LIMIT {
                                    format!("{:.*}{}/s", 1, converted_write.0, converted_write.1)
                                } else {
                                    format!("{:.*}{}/s", 0, converted_write.0, converted_write.1)
                                },
                            );
                        }
                    }
                } else {
                    if self.io_labels.len() <= itx {
                        self.io_labels.push((String::default(), String::default()));
                    }

                    if let Some(io_labels) = self.io_labels.get_mut(itx) {
                        *io_labels = ("N/A".to_string(), "N/A".to_string());
                    }
                }
            }
        }

        self.disk_harvest = disks;
        self.io_harvest = io;
    }

    fn eat_proc(&mut self, list_of_processes: Vec<processes::ProcessHarvest>) {
        self.process_data.ingest(list_of_processes);
    }

    #[cfg(feature = "battery")]
    fn eat_battery(&mut self, list_of_batteries: Vec<batteries::BatteryHarvest>) {
        self.battery_harvest = list_of_batteries;
    }
}
