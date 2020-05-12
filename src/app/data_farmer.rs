use lazy_static::lazy_static;
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
use std::{time::Instant, vec::Vec};

use crate::data_harvester::{
    battery_harvester, cpu, disks, mem, network, processes, temperature, Data,
};
use regex::Regex;

pub type TimeOffset = f64;
pub type Value = f64;

#[derive(Debug, Default)]
pub struct TimedData {
    pub rx_data: Value,
    pub tx_data: Value,
    pub cpu_data: Vec<Value>,
    pub mem_data: Value,
    pub swap_data: Value,
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
    pub cpu_harvest: cpu::CPUHarvest,
    pub process_harvest: Vec<processes::ProcessHarvest>,
    pub disk_harvest: Vec<disks::DiskHarvest>,
    pub io_harvest: disks::IOHarvest,
    pub io_labels_and_prev: Vec<((u64, u64), (u64, u64))>,
    pub temp_harvest: Vec<temperature::TempHarvest>,
    pub battery_harvest: Vec<battery_harvester::BatteryHarvest>,
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
            cpu_harvest: cpu::CPUHarvest::default(),
            process_harvest: Vec::default(),
            disk_harvest: Vec::default(),
            io_harvest: disks::IOHarvest::default(),
            io_labels_and_prev: Vec::default(),
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
        self.cpu_harvest = cpu::CPUHarvest::default();
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
        let current_time = Instant::now();

        let mut remove_index = 0;
        for entry in &self.timed_data_vec {
            if current_time.duration_since(entry.0).as_millis() >= max_time_millis as u128 {
                remove_index += 1;
            } else {
                break;
            }
        }

        self.timed_data_vec.drain(0..remove_index);
    }

    pub fn eat_data(&mut self, harvested_data: &Data) {
        let harvested_time = harvested_data.last_collection_time;
        let mut new_entry = TimedData::default();

        // Network
        if let Some(network) = &harvested_data.network {
            self.eat_network(network, &mut new_entry);
        }

        // Memory and Swap
        if let Some(memory) = &harvested_data.memory {
            if let Some(swap) = &harvested_data.swap {
                self.eat_memory_and_swap(memory, swap, &mut new_entry);
            }
        }

        // CPU
        if let Some(cpu) = &harvested_data.cpu {
            self.eat_cpu(cpu, &mut new_entry);
        }

        // Temp
        if let Some(temperature_sensors) = &harvested_data.temperature_sensors {
            self.eat_temp(temperature_sensors);
        }

        // Disks
        if let Some(disks) = &harvested_data.disks {
            if let Some(io) = &harvested_data.io {
                self.eat_disks(disks, io, harvested_time);
            }
        }

        // Processes
        if let Some(list_of_processes) = &harvested_data.list_of_processes {
            self.eat_proc(list_of_processes);
        }

        // Battery
        if let Some(list_of_batteries) = &harvested_data.list_of_batteries {
            self.eat_battery(list_of_batteries);
        }

        // And we're done eating.  Update time and push the new entry!
        self.current_instant = harvested_time;
        self.timed_data_vec.push((harvested_time, new_entry));
    }

    fn eat_memory_and_swap(
        &mut self, memory: &mem::MemHarvest, swap: &mem::MemHarvest, new_entry: &mut TimedData,
    ) {
        // Memory
        let mem_percent = match memory.mem_total_in_mb {
            0 => 0f64,
            total => (memory.mem_used_in_mb as f64) / (total as f64) * 100.0,
        };
        new_entry.mem_data = mem_percent;

        // Swap
        if swap.mem_total_in_mb > 0 {
            let swap_percent = match swap.mem_total_in_mb {
                0 => 0f64,
                total => (swap.mem_used_in_mb as f64) / (total as f64) * 100.0,
            };
            new_entry.swap_data = swap_percent;
        }

        // In addition copy over latest data for easy reference
        self.memory_harvest = memory.clone();
        self.swap_harvest = swap.clone();
    }

    fn eat_network(&mut self, network: &network::NetworkHarvest, new_entry: &mut TimedData) {
        // RX
        let logged_rx_val = if network.rx as f64 > 0.0 {
            (network.rx as f64).log(2.0)
        } else {
            0.0
        };
        new_entry.rx_data = logged_rx_val;

        // TX
        let logged_tx_val = if network.tx as f64 > 0.0 {
            (network.tx as f64).log(2.0)
        } else {
            0.0
        };
        new_entry.tx_data = logged_tx_val;

        // In addition copy over latest data for easy reference
        self.network_harvest = network.clone();
    }

    fn eat_cpu(&mut self, cpu: &[cpu::CPUData], new_entry: &mut TimedData) {
        // Note this only pre-calculates the data points - the names will be
        // within the local copy of cpu_harvest.  Since it's all sequential
        // it probably doesn't matter anyways.
        cpu.iter()
            .for_each(|cpu| new_entry.cpu_data.push(cpu.cpu_usage));

        self.cpu_harvest = cpu.to_vec();
    }

    fn eat_temp(&mut self, temperature_sensors: &[temperature::TempHarvest]) {
        // TODO: [PO] To implement
        self.temp_harvest = temperature_sensors.to_vec();
    }

    fn eat_disks(
        &mut self, disks: &[disks::DiskHarvest], io: &disks::IOHarvest, harvested_time: Instant,
    ) {
        // TODO: [PO] To implement

        let time_since_last_harvest = harvested_time
            .duration_since(self.current_instant)
            .as_secs_f64();

        for (itx, device) in disks.iter().enumerate() {
            if let Some(trim) = device.name.split('/').last() {
                let io_device = if cfg!(target_os = "macos") {
                    // Must trim one level further!

                    lazy_static! {
                        static ref DISK_REGEX: Regex = Regex::new(r"disk\d+").unwrap();
                    }
                    if let Some(disk_trim) = DISK_REGEX.split(trim).next() {
                        io.get(disk_trim)
                    } else {
                        None
                    }
                } else {
                    io.get(trim)
                };
                let (io_r_pt, io_w_pt) = if let Some(io) = io_device {
                    (io.read_bytes, io.write_bytes)
                } else {
                    (0, 0)
                };

                if self.io_labels_and_prev.len() <= itx {
                    self.io_labels_and_prev.push(((0, 0), (io_r_pt, io_w_pt)));
                } else if let Some((io_curr, io_prev)) = self.io_labels_and_prev.get_mut(itx) {
                    let r_rate = ((io_r_pt.saturating_sub(io_prev.0)) as f64
                        / time_since_last_harvest)
                        .round() as u64;
                    let w_rate = ((io_w_pt.saturating_sub(io_prev.1)) as f64
                        / time_since_last_harvest)
                        .round() as u64;

                    *io_curr = (r_rate, w_rate);
                    *io_prev = (io_r_pt, io_w_pt);
                }
            }
        }

        self.disk_harvest = disks.to_vec();
        self.io_harvest = io.clone();
    }

    fn eat_proc(&mut self, list_of_processes: &[processes::ProcessHarvest]) {
        self.process_harvest = list_of_processes.to_vec();
    }

    fn eat_battery(&mut self, list_of_batteries: &[battery_harvester::BatteryHarvest]) {
        self.battery_harvest = list_of_batteries.to_vec();
    }
}
