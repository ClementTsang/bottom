//! In charge of cleaning, processing, and managing data.

use std::{
    collections::BTreeMap,
    time::{Duration, Instant},
    vec::Vec,
};

use hashbrown::{HashMap, HashSet};
use timeless::data::ChunkedData;

#[cfg(feature = "battery")]
use crate::data_collection::batteries;
use crate::{
    data_collection::{
        cpu, disks, memory, network,
        processes::{Pid, ProcessHarvest},
        temperature, Data,
    },
    dec_bytes_per_second_string,
};

/// Values corresponding to a time slice.
type Values = ChunkedData<f64>;

/// A wrapper around a vector of lists of data.
struct ValueChunkList(Vec<Values>);

impl ValueChunkList {
    /// Prune all lists.
    pub fn prune(&mut self, remove_up_to: usize) {
        for v in &mut self.0 {
            v.prune(remove_up_to);
        }
    }
}

/// A wrapper around an [`Instant`], with the default being [`Instant::now`].
#[derive(Debug, Clone, Copy)]
struct DefaultInstant(Instant);

impl Default for DefaultInstant {
    fn default() -> Self {
        Self(Instant::now())
    }
}

/// Represents timeseries data in a chunked, deduped manner.
///
/// Properties:
/// - Time in this manner is represented in a reverse-offset fashion from the current time.
/// - All data is stored in SoA fashion.
/// - Values are stored in a chunked format, which facilitates gaps in data collection if needed.
/// - Additional metadata is stored to make data pruning over time easy.
#[derive(Debug, Default)]
pub struct TimeSeriesData {
    /// Time values.
    time: Vec<Instant>,

    /// Network RX data.
    rx: Values,

    /// Network TX data.
    tx: Values,

    /// CPU data.
    cpu: Vec<Values>,

    /// Memory data.
    mem: Values,

    /// Swap data.
    swap: Values,

    #[cfg(not(target_os = "windows"))]
    /// Cache data.
    cache_mem: Values,

    #[cfg(feature = "zfs")]
    /// Arc data.
    arc_mem: Values,

    #[cfg(feature = "gpu")]
    /// GPU memory data.
    gpu_mem: HashMap<String, Values>,
}

impl TimeSeriesData {
    /// Add a new data point.
    pub fn add(&mut self, data: Data) {
        self.time.push(data.collection_time);

        if let Some(network) = data.network {
            self.rx.push(network.rx as f64);
            self.tx.push(network.tx as f64);
        } else {
            self.rx.insert_break();
            self.tx.insert_break();
        }

        if let Some(cpu) = data.cpu {
            if self.cpu.len() < cpu.len() {
                let diff = cpu.len() - self.cpu.len();
                self.cpu.reserve_exact(diff);

                for _ in 0..diff {
                    self.cpu.push(Default::default());
                }
            } else if self.cpu.len() > cpu.len() {
                let diff = self.cpu.len() - cpu.len();
                let offset = self.cpu.len() - diff;

                for curr in &mut self.cpu[offset..] {
                    curr.insert_break();
                }
            }

            for (curr, new_data) in self.cpu.iter_mut().zip(cpu.into_iter()) {
                curr.push(new_data.cpu_usage);
            }
        } else {
            for c in &mut self.cpu {
                c.insert_break();
            }
        }

        if let Some(memory) = data.memory {
            self.mem.try_push(memory.checked_percent());
        } else {
            self.mem.insert_break();
        }

        if let Some(swap) = data.swap {
            self.swap.try_push(swap.checked_percent());
        } else {
            self.swap.insert_break();
        }

        #[cfg(not(target_os = "windows"))]
        {
            if let Some(cache) = data.cache {
                self.cache_mem.try_push(cache.checked_percent());
            } else {
                self.cache_mem.insert_break();
            }
        }

        #[cfg(feature = "zfs")]
        {
            if let Some(arc) = data.arc {
                self.arc_mem.try_push(arc.checked_percent());
            } else {
                self.arc_mem.insert_break();
            }
        }

        #[cfg(feature = "gpu")]
        {
            if let Some(gpu) = data.gpu {
                let mut not_visited = self
                    .gpu_mem
                    .keys()
                    .map(String::to_owned)
                    .collect::<HashSet<_>>();

                for (name, new_data) in gpu {
                    not_visited.remove(&name);
                    let curr = self.gpu_mem.entry(name).or_default();
                    curr.try_push(new_data.checked_percent());
                }

                for nv in not_visited {
                    if let Some(entry) = self.gpu_mem.get_mut(&nv) {
                        entry.insert_break();
                    }
                }
            } else {
                for g in self.gpu_mem.values_mut() {
                    g.insert_break();
                }
            }
        }
    }

    /// Prune any data older than the given duration.
    pub fn prune(&mut self, max_age: Duration) -> Result<(), usize> {
        if self.time.is_empty() {
            return Ok(());
        }

        let now = Instant::now();

        let end = match self
            .time
            .binary_search_by(|then| now.duration_since(*then).cmp(&max_age).reverse())
        {
            Ok(index) => index,
            Err(index) => index - 1, // Safe as length is > 0.
        };

        // Note that end here is _inclusive_.
        self.time.drain(0..=end);

        self.rx.prune(end)?;
        self.tx.prune(end)?;

        for cpu in &mut self.cpu {
            cpu.prune(end)?;

            // // We don't want to retain things if there is no data at all.
            // if cpu.is_empty() {
            //     to_delete.push(itx);
            // }
        }

        self.mem.prune(end)?;
        self.swap.prune(end)?;

        #[cfg(not(target_os = "windows"))]
        self.cache_mem.prune(end)?;

        #[cfg(feature = "zfs")]
        self.arc_mem.prune(end)?;

        #[cfg(feature = "gpu")]
        {
            for gpu in self.gpu_mem.values_mut() {
                gpu.prune(end)?;

                // // We don't want to retain things if there is no data at all.
                // if gpu.is_empty() {
                //     to_delete.push(itx);
                // }
            }
        }

        Ok(())
    }
}

#[derive(Debug, Default, Clone)]
pub struct TimedData {
    pub rx_data: f64,
    pub tx_data: f64,
    pub cpu_data: Vec<f64>,
    pub mem_data: Option<f64>,
    #[cfg(not(target_os = "windows"))]
    pub cache_data: Option<f64>,
    pub swap_data: Option<f64>,
    #[cfg(feature = "zfs")]
    pub arc_data: Option<f64>,
    #[cfg(feature = "gpu")]
    pub gpu_data: Vec<Option<f64>>,
}

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
    fn ingest(&mut self, list_of_processes: Vec<ProcessHarvest>) {
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

/// AppCollection represents the pooled data stored within the main app
/// thread.  Basically stores a (occasionally cleaned) record of the data
/// collected, and what is needed to convert into a displayable form.
///
/// If the app is *frozen* - that is, we do not want to *display* any changing
/// data, keep updating this. As of 2021-09-08, we just clone the current
/// collection when it freezes to have a snapshot floating around.
///
/// Note that with this method, the *app* thread is responsible for cleaning -
/// not the data collector.
#[derive(Debug, Clone)]
pub struct DataCollection {
    pub current_instant: Instant,
    pub timed_data_vec: Vec<(Instant, TimedData)>,
    pub network_harvest: network::NetworkHarvest,
    pub memory_harvest: memory::MemHarvest,
    #[cfg(not(target_os = "windows"))]
    pub cache_harvest: memory::MemHarvest,
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
    pub battery_harvest: Vec<batteries::BatteryData>,
    #[cfg(feature = "zfs")]
    pub arc_harvest: memory::MemHarvest,
    #[cfg(feature = "gpu")]
    pub gpu_harvest: Vec<(String, memory::MemHarvest)>,
}

impl Default for DataCollection {
    fn default() -> Self {
        DataCollection {
            current_instant: Instant::now(),
            timed_data_vec: Vec::default(),
            network_harvest: network::NetworkHarvest::default(),
            memory_harvest: memory::MemHarvest::default(),
            #[cfg(not(target_os = "windows"))]
            cache_harvest: memory::MemHarvest::default(),
            swap_harvest: memory::MemHarvest::default(),
            cpu_harvest: cpu::CpuHarvest::default(),
            load_avg_harvest: cpu::LoadAvgHarvest::default(),
            process_data: Default::default(),
            disk_harvest: Vec::default(),
            io_harvest: disks::IoHarvest::default(),
            io_labels_and_prev: Vec::default(),
            io_labels: Vec::default(),
            temp_harvest: Vec::default(),
            #[cfg(feature = "battery")]
            battery_harvest: Vec::default(),
            #[cfg(feature = "zfs")]
            arc_harvest: memory::MemHarvest::default(),
            #[cfg(feature = "gpu")]
            gpu_harvest: Vec::default(),
        }
    }
}

impl DataCollection {
    pub fn reset(&mut self) {
        self.timed_data_vec = Vec::default();
        self.network_harvest = network::NetworkHarvest::default();
        self.memory_harvest = memory::MemHarvest::default();
        self.swap_harvest = memory::MemHarvest::default();
        self.cpu_harvest = cpu::CpuHarvest::default();
        self.process_data = Default::default();
        self.disk_harvest = Vec::default();
        self.io_harvest = disks::IoHarvest::default();
        self.io_labels_and_prev = Vec::default();
        self.temp_harvest = Vec::default();
        #[cfg(feature = "battery")]
        {
            self.battery_harvest = Vec::default();
        }
        #[cfg(feature = "zfs")]
        {
            self.arc_harvest = memory::MemHarvest::default();
        }
        #[cfg(feature = "gpu")]
        {
            self.gpu_harvest = Vec::default();
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
                    .cmp(&(max_time_millis.into()))
                    .reverse()
            }) {
            Ok(index) => index,
            Err(index) => index,
        };

        self.timed_data_vec.drain(0..remove_index);
        self.timed_data_vec.shrink_to_fit();
    }

    #[allow(
        clippy::boxed_local,
        reason = "Clippy allow to avoid warning on certain platforms (e.g. 32-bit)."
    )]
    pub fn eat_data(&mut self, harvested_data: Box<Data>) {
        let harvested_time = harvested_data.collection_time;
        let mut new_entry = TimedData::default();

        // Network
        if let Some(network) = harvested_data.network {
            self.eat_network(network, &mut new_entry);
        }

        // Memory, Swap
        if let (Some(memory), Some(swap)) = (harvested_data.memory, harvested_data.swap) {
            self.eat_memory_and_swap(memory, swap, &mut new_entry);
        }

        // Cache memory
        #[cfg(not(target_os = "windows"))]
        if let Some(cache) = harvested_data.cache {
            self.eat_cache(cache, &mut new_entry);
        }

        #[cfg(feature = "zfs")]
        if let Some(arc) = harvested_data.arc {
            self.eat_arc(arc, &mut new_entry);
        }

        #[cfg(feature = "gpu")]
        if let Some(gpu) = harvested_data.gpu {
            self.eat_gpu(gpu, &mut new_entry);
        }

        // CPU
        if let Some(cpu) = harvested_data.cpu {
            self.eat_cpu(cpu, &mut new_entry);
        }

        // Load average
        if let Some(load_avg) = harvested_data.load_avg {
            self.eat_load_avg(load_avg);
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
        new_entry.mem_data = memory.checked_percent();
        new_entry.swap_data = swap.checked_percent();

        // In addition copy over latest data for easy reference
        self.memory_harvest = memory;
        self.swap_harvest = swap;
    }

    #[cfg(not(target_os = "windows"))]
    fn eat_cache(&mut self, cache: memory::MemHarvest, new_entry: &mut TimedData) {
        new_entry.cache_data = cache.checked_percent();
        self.cache_harvest = cache;
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

        self.cpu_harvest = cpu;
    }

    fn eat_load_avg(&mut self, load_avg: cpu::LoadAvgHarvest) {
        self.load_avg_harvest = load_avg;
    }

    fn eat_temp(&mut self, temperature_sensors: Vec<temperature::TempHarvest>) {
        self.temp_harvest = temperature_sensors;
    }

    fn eat_disks(
        &mut self, disks: Vec<disks::DiskHarvest>, io: disks::IoHarvest, harvested_time: Instant,
    ) {
        let time_since_last_harvest = harvested_time
            .duration_since(self.current_instant)
            .as_secs_f64();

        for (itx, device) in disks.iter().enumerate() {
            let checked_name = {
                #[cfg(target_os = "windows")]
                {
                    match &device.volume_name {
                        Some(volume_name) => Some(volume_name.as_str()),
                        None => device.name.split('/').last(),
                    }
                }
                #[cfg(not(target_os = "windows"))]
                {
                    #[cfg(feature = "zfs")]
                    {
                        if !device.name.starts_with('/') {
                            Some(device.name.as_str()) // use the whole zfs
                                                       // dataset name
                        } else {
                            device.name.split('/').last()
                        }
                    }
                    #[cfg(not(feature = "zfs"))]
                    {
                        device.name.split('/').last()
                    }
                }
            };

            if let Some(checked_name) = checked_name {
                let io_device = {
                    #[cfg(target_os = "macos")]
                    {
                        use std::sync::OnceLock;

                        use regex::Regex;

                        // Must trim one level further for macOS!
                        static DISK_REGEX: OnceLock<Regex> = OnceLock::new();

                        #[expect(
                            clippy::regex_creation_in_loops,
                            reason = "this is fine since it's done via a static OnceLock. In the future though, separate it out."
                        )]
                        if let Some(new_name) = DISK_REGEX
                            .get_or_init(|| Regex::new(r"disk\d+").unwrap())
                            .find(checked_name)
                        {
                            io.get(new_name.as_str())
                        } else {
                            None
                        }
                    }
                    #[cfg(not(target_os = "macos"))]
                    {
                        io.get(checked_name)
                    }
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

                        // TODO: idk why I'm generating this here tbh
                        if let Some(io_labels) = self.io_labels.get_mut(itx) {
                            *io_labels = (
                                dec_bytes_per_second_string(r_rate),
                                dec_bytes_per_second_string(w_rate),
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

    fn eat_proc(&mut self, list_of_processes: Vec<ProcessHarvest>) {
        self.process_data.ingest(list_of_processes);
    }

    #[cfg(feature = "battery")]
    fn eat_battery(&mut self, list_of_batteries: Vec<batteries::BatteryData>) {
        self.battery_harvest = list_of_batteries;
    }

    #[cfg(feature = "zfs")]
    fn eat_arc(&mut self, arc: memory::MemHarvest, new_entry: &mut TimedData) {
        new_entry.arc_data = arc.checked_percent();
        self.arc_harvest = arc;
    }

    #[cfg(feature = "gpu")]
    fn eat_gpu(&mut self, gpu: Vec<(String, memory::MemHarvest)>, new_entry: &mut TimedData) {
        // Note this only pre-calculates the data points - the names will be
        // within the local copy of gpu_harvest. Since it's all sequential
        // it probably doesn't matter anyways.
        gpu.iter().for_each(|data| {
            new_entry.gpu_data.push(data.1.checked_percent());
        });
        self.gpu_harvest = gpu;
    }
}
