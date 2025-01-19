//! In charge of cleaning, processing, and managing data.

use std::{
    cmp::Ordering,
    collections::BTreeMap,
    time::{Duration, Instant},
    vec::Vec,
};

use hashbrown::{HashMap, HashSet}; // TODO: Try fxhash again.
use timeless::data::ChunkedData;

#[cfg(feature = "battery")]
use crate::data_collection::batteries;
use crate::{
    data_collection::{
        cpu, disks,
        memory::MemHarvest,
        network,
        processes::{Pid, ProcessHarvest},
        Data,
    },
    dec_bytes_per_second_string,
    widgets::TempWidgetData,
};

/// Values corresponding to a time slice.
pub type Values = ChunkedData<f64>;

/// Represents timeseries data in a chunked, deduped manner.
///
/// Properties:
/// - Time in this manner is represented in a reverse-offset fashion from the current time.
/// - All data is stored in SoA fashion.
/// - Values are stored in a chunked format, which facilitates gaps in data collection if needed.
/// - Additional metadata is stored to make data pruning over time easy.
#[derive(Clone, Debug, Default)]
pub struct TimeSeriesData {
    /// Time values.
    pub time: Vec<Instant>,

    /// Network RX data.
    pub rx: Values,

    /// Network TX data.
    pub tx: Values,

    /// CPU data.
    pub cpu: Vec<Values>,

    /// RAM memory data.
    pub ram: Values,

    /// Swap data.
    pub swap: Values,

    #[cfg(not(target_os = "windows"))]
    /// Cache data.
    pub cache_mem: Values,

    #[cfg(feature = "zfs")]
    /// Arc data.
    pub arc_mem: Values,

    #[cfg(feature = "gpu")]
    /// GPU memory data.
    pub gpu_mem: HashMap<String, Values>,
}

impl TimeSeriesData {
    /// Add a new data point.
    pub fn add(&mut self, data: &Data) {
        self.time.push(data.collection_time);

        if let Some(network) = &data.network {
            self.rx.push(network.rx as f64);
            self.tx.push(network.tx as f64);
        } else {
            self.rx.insert_break();
            self.tx.insert_break();
        }

        if let Some(cpu) = &data.cpu {
            match self.cpu.len().cmp(&cpu.len()) {
                Ordering::Less => {
                    let diff = cpu.len() - self.cpu.len();
                    self.cpu.reserve_exact(diff);

                    for _ in 0..diff {
                        self.cpu.push(Default::default());
                    }
                }
                Ordering::Greater => {
                    let diff = self.cpu.len() - cpu.len();
                    let offset = self.cpu.len() - diff;

                    for curr in &mut self.cpu[offset..] {
                        curr.insert_break();
                    }
                }
                Ordering::Equal => {}
            }

            for (curr, new_data) in self.cpu.iter_mut().zip(cpu.iter()) {
                curr.push(new_data.cpu_usage);
            }
        } else {
            for c in &mut self.cpu {
                c.insert_break();
            }
        }

        if let Some(memory) = &data.memory {
            self.ram.try_push(memory.checked_percent());
        } else {
            self.ram.insert_break();
        }

        if let Some(swap) = &data.swap {
            self.swap.try_push(swap.checked_percent());
        } else {
            self.swap.insert_break();
        }

        #[cfg(not(target_os = "windows"))]
        {
            if let Some(cache) = &data.cache {
                self.cache_mem.try_push(cache.checked_percent());
            } else {
                self.cache_mem.insert_break();
            }
        }

        #[cfg(feature = "zfs")]
        {
            if let Some(arc) = &data.arc {
                self.arc_mem.try_push(arc.checked_percent());
            } else {
                self.arc_mem.insert_break();
            }
        }

        #[cfg(feature = "gpu")]
        {
            if let Some(gpu) = &data.gpu {
                let mut not_visited = self
                    .gpu_mem
                    .keys()
                    .map(String::to_owned)
                    .collect::<HashSet<_>>();

                for (name, new_data) in gpu {
                    not_visited.remove(name);

                    if !self.gpu_mem.contains_key(name) {
                        self.gpu_mem
                            .insert(name.to_string(), ChunkedData::default());
                    }

                    let curr = self
                        .gpu_mem
                        .get_mut(name)
                        .expect("entry must exist as it was created above");
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
    pub fn prune(&mut self, max_age: Duration) {
        if self.time.is_empty() {
            return;
        }

        let now = Instant::now();
        let end = {
            let partition_point = self
                .time
                .partition_point(|then| now.duration_since(*then) > max_age);

            // Partition point returns the first index that does not match the predicate, so minus one.
            if partition_point > 0 {
                partition_point - 1
            } else {
                // If the partition point was 0, then it means all values are too new to be pruned.
                crate::info!("Skipping prune.");
                return;
            }
        };

        crate::info!("Pruning up to index {end}.");

        // Note that end here is _inclusive_.
        self.time.drain(0..=end);
        self.time.shrink_to_fit();

        let _ = self.rx.prune(end);
        let _ = self.tx.prune(end);

        for cpu in &mut self.cpu {
            let _ = cpu.prune(end);
        }

        let _ = self.ram.prune(end);
        let _ = self.swap.prune(end);

        #[cfg(not(target_os = "windows"))]
        let _ = self.cache_mem.prune(end);

        #[cfg(feature = "zfs")]
        let _ = self.arc_mem.prune(end);

        #[cfg(feature = "gpu")]
        {
            for gpu in self.gpu_mem.values_mut() {
                let _ = gpu.prune(end);

                // TODO: Do we want to filter out any empty gpus?
            }
        }
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
pub struct CollectedData {
    pub current_instant: Instant,
    pub timed_data_vec: Vec<(Instant, TimedData)>, // FIXME: (points_rework_v1) REMOVE THIS
    pub timeseries_data: TimeSeriesData,           // FIXME: (points_rework_v1) Skip in basic?
    pub network_harvest: network::NetworkHarvest,
    pub ram_harvest: MemHarvest,
    pub swap_harvest: Option<MemHarvest>,
    #[cfg(not(target_os = "windows"))]
    pub cache_harvest: Option<MemHarvest>,
    #[cfg(feature = "zfs")]
    pub arc_harvest: Option<MemHarvest>,
    #[cfg(feature = "gpu")]
    pub gpu_harvest: Vec<(String, MemHarvest)>,
    pub cpu_harvest: cpu::CpuHarvest,
    pub load_avg_harvest: cpu::LoadAvgHarvest,
    pub process_data: ProcessData,
    pub disk_harvest: Vec<disks::DiskHarvest>,
    // TODO: The IO labels are kinda weird.
    pub io_labels_and_prev: Vec<((u64, u64), (u64, u64))>,
    pub io_labels: Vec<(String, String)>,
    pub temp_data: Vec<TempWidgetData>,
    #[cfg(feature = "battery")]
    pub battery_harvest: Vec<batteries::BatteryData>,
}

impl Default for CollectedData {
    fn default() -> Self {
        CollectedData {
            current_instant: Instant::now(),
            timed_data_vec: Vec::default(),
            timeseries_data: TimeSeriesData::default(),
            network_harvest: network::NetworkHarvest::default(),
            ram_harvest: MemHarvest::default(),
            #[cfg(not(target_os = "windows"))]
            cache_harvest: None,
            swap_harvest: None,
            cpu_harvest: cpu::CpuHarvest::default(),
            load_avg_harvest: cpu::LoadAvgHarvest::default(),
            process_data: Default::default(),
            disk_harvest: Vec::default(),
            io_labels_and_prev: Vec::default(),
            io_labels: Vec::default(),
            temp_data: Vec::default(),
            #[cfg(feature = "battery")]
            battery_harvest: Vec::default(),
            #[cfg(feature = "zfs")]
            arc_harvest: None,
            #[cfg(feature = "gpu")]
            gpu_harvest: Vec::default(),
        }
    }
}

impl CollectedData {
    pub fn reset(&mut self) {
        *self = CollectedData::default();
    }

    #[allow(
        clippy::boxed_local,
        reason = "This avoids warnings on certain platforms (e.g. 32-bit)."
    )]
    fn eat_data(&mut self, data: Box<Data>) {
        let harvested_time = data.collection_time;

        self.timeseries_data.add(&data);

        // Network
        if let Some(network) = data.network {
            self.network_harvest = network;
        }

        // Memory, Swap
        if let Some(memory) = data.memory {
            self.ram_harvest = memory;
        }

        self.swap_harvest = data.swap;

        // Cache memory
        #[cfg(not(target_os = "windows"))]
        {
            self.cache_harvest = data.cache;
        }

        #[cfg(feature = "zfs")]
        {
            self.arc_harvest = data.arc;
        }

        #[cfg(feature = "gpu")]
        if let Some(gpu) = data.gpu {
            self.gpu_harvest = gpu;
        }

        // CPU
        if let Some(cpu) = data.cpu {
            self.cpu_harvest = cpu;
        }

        // Load average
        if let Some(load_avg) = data.load_avg {
            self.load_avg_harvest = load_avg;
        }

        // Temp
        // TODO: (points_rework_v1) the map might be redundant, the types are the same.
        self.temp_data = data
            .temperature_sensors
            .map(|sensors| {
                sensors
                    .into_iter()
                    .map(|temp| TempWidgetData {
                        sensor: temp.name,
                        temperature: temp.temperature.map(|v| v.into()),
                    })
                    .collect()
            })
            .unwrap_or_default();

        // Disks
        if let Some(disks) = data.disks {
            if let Some(io) = data.io {
                self.eat_disks(disks, io, harvested_time);
            }
        }

        // Processes
        if let Some(list_of_processes) = data.list_of_processes {
            self.process_data.ingest(list_of_processes);
        }

        #[cfg(feature = "battery")]
        {
            // Battery
            if let Some(list_of_batteries) = data.list_of_batteries {
                self.battery_harvest = list_of_batteries;
            }
        }

        // And we're done eating.  Update time and push the new entry!
        self.current_instant = harvested_time;
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
    }
}

/// If we freeze data collection updates, we want to return a "frozen" copy
/// of the data at the time, while still updating things in the background.
#[derive(Default)]
pub enum FrozenState {
    #[default]
    NotFrozen,
    Frozen(Box<CollectedData>),
}

/// What data to share to other parts of the application.
#[derive(Default)]
pub struct DataStore {
    frozen_state: FrozenState,
    main: CollectedData,
}

impl DataStore {
    /// Toggle whether the [`DataState`] is frozen or not.
    pub fn toggle_frozen(&mut self) {
        match &self.frozen_state {
            FrozenState::NotFrozen => {
                self.frozen_state = FrozenState::Frozen(Box::new(self.main.clone()));
            }
            FrozenState::Frozen(_) => self.frozen_state = FrozenState::NotFrozen,
        }
    }

    /// Return whether the [`DataState`] is frozen or not.
    pub fn is_frozen(&self) -> bool {
        matches!(self.frozen_state, FrozenState::Frozen(_))
    }

    /// Return a reference to the current [`DataCollection`] based on state.
    pub fn get_data(&self) -> &CollectedData {
        match &self.frozen_state {
            FrozenState::NotFrozen => &self.main,
            FrozenState::Frozen(collected_data) => collected_data,
        }
    }

    /// Eat data.
    pub fn eat_data(&mut self, data: Box<Data>) {
        self.main.eat_data(data);
    }

    /// Clean data.
    pub fn clean_data(&mut self, max_duration: Duration) {
        self.main.timeseries_data.prune(max_duration);
    }

    /// Reset data state.
    pub fn reset(&mut self) {
        self.frozen_state = FrozenState::NotFrozen;
        self.main = CollectedData::default();
    }
}
