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
use std::time::Instant;
use std::vec::Vec;

use crate::data_harvester::{cpu, disks, mem, network, processes, temperature, Data};

pub type TimeOffset = f64;
pub type Value = f64;
pub type JoinedDataPoints = (Value, Vec<(TimeOffset, Value)>);

#[derive(Debug, Default)]
pub struct TimedData {
    pub rx_data: JoinedDataPoints,
    pub tx_data: JoinedDataPoints,
    pub cpu_data: Vec<JoinedDataPoints>,
    pub mem_data: JoinedDataPoints,
    pub swap_data: JoinedDataPoints,
    // Unused for now
    // pub io_data : JoinedDataPoints
    // pub temp_data: JoinedDataPoints,
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
    pub timed_data_vec: Vec<(Instant, TimedData)>,
    pub network_harvest: network::NetworkHarvest,
    pub memory_harvest: mem::MemHarvest,
    pub swap_harvest: mem::MemHarvest,
    pub cpu_harvest: cpu::CPUHarvest,
    pub process_harvest: Vec<processes::ProcessHarvest>,
    pub disk_harvest: Vec<disks::DiskHarvest>,
    pub io_harvest: disks::IOHarvest,
    pub io_labels: Vec<(u64, u64)>,
    io_prev: Vec<(u64, u64)>,
    pub temp_harvest: Vec<temperature::TempHarvest>,
}

impl Default for DataCollection {
    fn default() -> Self {
        DataCollection {
            current_instant: Instant::now(),
            timed_data_vec: Vec::default(),
            network_harvest: network::NetworkHarvest::default(),
            memory_harvest: mem::MemHarvest::default(),
            swap_harvest: mem::MemHarvest::default(),
            cpu_harvest: cpu::CPUHarvest::default(),
            process_harvest: Vec::default(),
            disk_harvest: Vec::default(),
            io_harvest: disks::IOHarvest::default(),
            io_labels: Vec::default(),
            io_prev: Vec::default(),
            temp_harvest: Vec::default(),
        }
    }
}

impl DataCollection {
    pub fn clean_data(&mut self, max_time_millis: u128) {
        let current_time = Instant::now();

        let mut remove_index = 0;
        for entry in &self.timed_data_vec {
            if current_time.duration_since(entry.0).as_millis() >= max_time_millis {
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
        self.eat_network(&harvested_data, harvested_time, &mut new_entry);

        // Memory and Swap
        self.eat_memory_and_swap(&harvested_data, harvested_time, &mut new_entry);

        // CPU
        self.eat_cpu(&harvested_data, harvested_time, &mut new_entry);

        // Temp
        self.eat_temp(&harvested_data);

        // Disks
        self.eat_disks(&harvested_data, harvested_time);

        // Processes
        self.eat_proc(&harvested_data);

        // And we're done eating.  Update time and push the new entry!
        self.current_instant = harvested_time;
        self.timed_data_vec.push((harvested_time, new_entry));
    }

    fn eat_memory_and_swap(
        &mut self, harvested_data: &Data, harvested_time: Instant, new_entry: &mut TimedData,
    ) {
        // Memory
        let mem_percent = harvested_data.memory.mem_used_in_mb as f64
            / harvested_data.memory.mem_total_in_mb as f64
            * 100.0;
        let mem_joining_pts = if let Some((time, last_pt)) = self.timed_data_vec.last() {
            generate_joining_points(*time, last_pt.mem_data.0, harvested_time, mem_percent)
        } else {
            Vec::new()
        };
        let mem_pt = (mem_percent, mem_joining_pts);
        new_entry.mem_data = mem_pt;

        // Swap
        if harvested_data.swap.mem_total_in_mb > 0 {
            let swap_percent = harvested_data.swap.mem_used_in_mb as f64
                / harvested_data.swap.mem_total_in_mb as f64
                * 100.0;
            let swap_joining_pt = if let Some((time, last_pt)) = self.timed_data_vec.last() {
                generate_joining_points(*time, last_pt.swap_data.0, harvested_time, swap_percent)
            } else {
                Vec::new()
            };
            let swap_pt = (swap_percent, swap_joining_pt);
            new_entry.swap_data = swap_pt;
        }

        // In addition copy over latest data for easy reference
        self.memory_harvest = harvested_data.memory.clone();
        self.swap_harvest = harvested_data.swap.clone();
    }

    fn eat_network(
        &mut self, harvested_data: &Data, harvested_time: Instant, new_entry: &mut TimedData,
    ) {
        // RX
        let logged_rx_val = if harvested_data.network.rx as f64 > 0.0 {
            (harvested_data.network.rx as f64).log(2.0)
        } else {
            0.0
        };

        let rx_joining_pts = if let Some((time, last_pt)) = self.timed_data_vec.last() {
            generate_joining_points(*time, last_pt.rx_data.0, harvested_time, logged_rx_val)
        } else {
            Vec::new()
        };
        let rx_pt = (logged_rx_val, rx_joining_pts);
        new_entry.rx_data = rx_pt;

        // TX
        let logged_tx_val = if harvested_data.network.tx as f64 > 0.0 {
            (harvested_data.network.tx as f64).log(2.0)
        } else {
            0.0
        };

        let tx_joining_pts = if let Some((time, last_pt)) = self.timed_data_vec.last() {
            generate_joining_points(*time, last_pt.tx_data.0, harvested_time, logged_tx_val)
        } else {
            Vec::new()
        };
        let tx_pt = (logged_tx_val, tx_joining_pts);
        new_entry.tx_data = tx_pt;

        // In addition copy over latest data for easy reference
        self.network_harvest = harvested_data.network.clone();
    }

    fn eat_cpu(
        &mut self, harvested_data: &Data, harvested_time: Instant, new_entry: &mut TimedData,
    ) {
        // Note this only pre-calculates the data points - the names will be
        // within the local copy of cpu_harvest.  Since it's all sequential
        // it probably doesn't matter anyways.
        for (itx, cpu) in harvested_data.cpu.iter().enumerate() {
            let cpu_joining_pts = if let Some((time, last_pt)) = self.timed_data_vec.last() {
                generate_joining_points(
                    *time,
                    last_pt.cpu_data[itx].0,
                    harvested_time,
                    cpu.cpu_usage,
                )
            } else {
                Vec::new()
            };

            let cpu_pt = (cpu.cpu_usage, cpu_joining_pts);
            new_entry.cpu_data.push(cpu_pt);
        }

        self.cpu_harvest = harvested_data.cpu.clone();
    }

    fn eat_temp(&mut self, harvested_data: &Data) {
        // TODO: [PO] To implement
        self.temp_harvest = harvested_data.temperature_sensors.clone();
    }

    fn eat_disks(&mut self, harvested_data: &Data, harvested_time: Instant) {
        // TODO: [PO] To implement

        let time_since_last_harvest = harvested_time
            .duration_since(self.current_instant)
            .as_secs_f64();

        for (itx, device) in harvested_data.disks.iter().enumerate() {
            if let Some(trim) = device.name.split('/').last() {
                let io_device = harvested_data.io.get(trim);
                if let Some(io) = io_device {
                    let io_r_pt = io.read_bytes;
                    let io_w_pt = io.write_bytes;

                    if self.io_labels.len() <= itx {
                        self.io_prev.push((io_r_pt, io_w_pt));
                        self.io_labels.push((0, 0));
                    } else {
                        let r_rate = ((io_r_pt - self.io_prev[itx].0) as f64
                            / time_since_last_harvest)
                            .round() as u64;
                        let w_rate = ((io_w_pt - self.io_prev[itx].1) as f64
                            / time_since_last_harvest)
                            .round() as u64;

                        self.io_labels[itx] = (r_rate, w_rate);
                        self.io_prev[itx] = (io_r_pt, io_w_pt);
                    }
                }
            }
        }

        self.disk_harvest = harvested_data.disks.clone();
        self.io_harvest = harvested_data.io.clone();
    }

    fn eat_proc(&mut self, harvested_data: &Data) {
        self.process_harvest = harvested_data.list_of_processes.clone();
    }
}

pub fn generate_joining_points(
    start_x: Instant, start_y: f64, end_x: Instant, end_y: f64,
) -> Vec<(TimeOffset, Value)> {
    let mut points: Vec<(TimeOffset, Value)> = Vec::new();

    // Convert time floats first:
    let tmp_time_diff = (end_x).duration_since(start_x).as_millis() as f64;
    let time_difference = if tmp_time_diff == 0.0 {
        0.001
    } else {
        tmp_time_diff
    };
    let value_difference = end_y - start_y;

    // Let's generate... about this many points!
    let num_points = std::cmp::min(
        std::cmp::max(
            (value_difference.abs() / time_difference * 2000.0) as u64,
            50,
        ),
        2000,
    );

    for itx in (0..num_points).step_by(2) {
        points.push((
            time_difference - (itx as f64 / num_points as f64 * time_difference),
            start_y + (itx as f64 / num_points as f64 * value_difference),
        ));
    }

    points
}
