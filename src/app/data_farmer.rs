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

use std::{time::Instant, vec::Vec};

use crate::{
    data_harvester::{batteries, cpu, disks, mem, network, processes, temperature, Data},
    utils::gen_util::get_simple_byte_values,
};
use regex::Regex;

pub type TimeOffset = f64;
pub type Value = f64;

#[derive(Debug, Default)]
pub struct TimedData {
    pub rx_data: Value,
    pub tx_data: Value,
    pub cpu_data: Vec<Value>,
    pub mem_data: Option<Value>,
    pub swap_data: Option<Value>,
}

/// AppCollection represents the pooled data stored within the main app
/// thread.  Basically stores a (occasionally cleaned) record of the data
/// collected, and what is needed to convert into a displayable form.
///
/// If the app is *frozen* - that is, we do not want to *display* any changing
/// data, keep updating this, don't convert to canvas displayable data!
///
/// Note that with this method, the *app* thread is responsible for cleaning -
/// not the data collector.
#[derive(Debug)]
pub struct DataCollection {
    pub current_instant: Instant,
    pub frozen_instant: Option<Instant>,
    pub timed_data_vec: Vec<(Instant, TimedData)>,
    pub network_harvest: network::NetworkHarvest,
    pub memory_harvest: mem::MemHarvest,
    pub swap_harvest: mem::MemHarvest,
    pub cpu_harvest: cpu::CpuHarvest,
    pub process_harvest: Vec<processes::ProcessHarvest>,
    pub disk_harvest: Vec<disks::DiskHarvest>,
    pub io_harvest: disks::IOHarvest,
    pub io_labels_and_prev: Vec<((u64, u64), (u64, u64))>,
    pub io_labels: Vec<(String, String)>,
    pub temp_harvest: Vec<temperature::TempHarvest>,
    pub battery_harvest: Vec<batteries::BatteryHarvest>,
}

impl Default for DataCollection {
    fn default() -> Self {
        DataCollection {
            current_instant: Instant::now(),
            frozen_instant: None,
            timed_data_vec: Vec::default(),
            network_harvest: network::NetworkHarvest::default(),
            memory_harvest: mem::MemHarvest::default(),
            swap_harvest: mem::MemHarvest::default(),
            cpu_harvest: cpu::CpuHarvest::default(),
            process_harvest: Vec::default(),
            disk_harvest: Vec::default(),
            io_harvest: disks::IOHarvest::default(),
            io_labels_and_prev: Vec::default(),
            io_labels: Vec::default(),
            temp_harvest: Vec::default(),
            battery_harvest: Vec::default(),
        }
    }
}

impl DataCollection {
    pub fn reset(&mut self) {
        self.timed_data_vec = Vec::default();
        self.network_harvest = network::NetworkHarvest::default();
        self.memory_harvest = mem::MemHarvest::default();
        self.swap_harvest = mem::MemHarvest::default();
        self.cpu_harvest = cpu::CpuHarvest::default();
        self.process_harvest = Vec::default();
        self.disk_harvest = Vec::default();
        self.io_harvest = disks::IOHarvest::default();
        self.io_labels_and_prev = Vec::default();
        self.temp_harvest = Vec::default();
        self.battery_harvest = Vec::default();
    }

    pub fn set_frozen_time(&mut self) {
        self.frozen_instant = Some(self.current_instant);
    }

    pub fn clean_data(&mut self, max_time_millis: u64) {
        // trace!("Cleaning data.");
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
        // trace!("Eating data now...");
        let harvested_time = harvested_data.last_collection_time;
        // trace!("Harvested time: {:?}", harvested_time);
        // trace!("New current instant: {:?}", self.current_instant);
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

        // Battery
        if let Some(list_of_batteries) = harvested_data.list_of_batteries {
            self.eat_battery(list_of_batteries);
        }

        // And we're done eating.  Update time and push the new entry!
        self.current_instant = harvested_time;
        self.timed_data_vec.push((harvested_time, new_entry));
    }

    fn eat_memory_and_swap(
        &mut self, memory: mem::MemHarvest, swap: mem::MemHarvest, new_entry: &mut TimedData,
    ) {
        // trace!("Eating mem and swap.");
        // Memory
        let mem_percent = if memory.mem_total_in_kb > 0 {
            Some((memory.mem_used_in_kb as f64) / (memory.mem_total_in_kb as f64) * 100.0)
        } else {
            None
        };
        new_entry.mem_data = mem_percent;

        // Swap
        let swap_percent = if swap.mem_total_in_kb > 0 {
            Some((swap.mem_used_in_kb as f64) / (swap.mem_total_in_kb as f64) * 100.0)
        } else {
            None
        };
        new_entry.swap_data = swap_percent;

        // In addition copy over latest data for easy reference
        self.memory_harvest = memory;
        self.swap_harvest = swap;
    }

    fn eat_network(&mut self, network: network::NetworkHarvest, new_entry: &mut TimedData) {
        // trace!("Eating network.");
        // FIXME [NETWORKING; CONFIG]: The ability to config this?
        // FIXME [NETWORKING]: Support bits, support switching between decimal and binary units (move the log part to conversion and switch on the fly)
        // RX
        new_entry.rx_data = if network.rx > 0 {
            (network.rx as f64).log2()
        } else {
            0.0
        };

        // TX
        new_entry.tx_data = if network.tx > 0 {
            (network.tx as f64).log2()
        } else {
            0.0
        };

        // In addition copy over latest data for easy reference
        self.network_harvest = network;
    }

    fn eat_cpu(&mut self, cpu: Vec<cpu::CpuData>, new_entry: &mut TimedData) {
        // trace!("Eating CPU.");
        // Note this only pre-calculates the data points - the names will be
        // within the local copy of cpu_harvest.  Since it's all sequential
        // it probably doesn't matter anyways.
        cpu.iter()
            .for_each(|cpu| new_entry.cpu_data.push(cpu.cpu_usage));

        self.cpu_harvest = cpu.to_vec();
    }

    fn eat_temp(&mut self, temperature_sensors: Vec<temperature::TempHarvest>) {
        // trace!("Eating temps.");
        // TODO: [PO] To implement
        self.temp_harvest = temperature_sensors.to_vec();
    }

    fn eat_disks(
        &mut self, disks: Vec<disks::DiskHarvest>, io: disks::IOHarvest, harvested_time: Instant,
    ) {
        // trace!("Eating disks.");
        // TODO: [PO] To implement

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
                            let converted_read = get_simple_byte_values(r_rate, false);
                            let converted_write = get_simple_byte_values(w_rate, false);
                            *io_labels = (
                                format!("{:.*}{}/s", 0, converted_read.0, converted_read.1),
                                format!("{:.*}{}/s", 0, converted_write.0, converted_write.1),
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
        // trace!("Eating proc.");
        self.process_harvest = list_of_processes;
    }

    fn eat_battery(&mut self, list_of_batteries: Vec<batteries::BatteryHarvest>) {
        // trace!("Eating batteries.");
        self.battery_harvest = list_of_batteries;
    }
}
