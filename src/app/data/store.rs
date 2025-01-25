use std::{
    time::{Duration, Instant},
    vec::Vec,
};

#[cfg(feature = "battery")]
use crate::collection::batteries;
use crate::{
    app::AppConfigFields,
    collection::{cpu, disks, memory::MemHarvest, network, Data},
    dec_bytes_per_second_string,
    widgets::TempWidgetData,
};

use super::{ProcessData, TimeSeriesData};

/// A collection of data. This is where we dump data into.
///
/// TODO: Maybe reduce visibility of internal data, make it only accessible through DataStore?
#[derive(Debug, Clone)]
pub struct StoredData {
    pub last_update_time: Instant, // FIXME: (points_rework_v1) remove this?
    pub timeseries_data: TimeSeriesData, // FIXME: (points_rework_v1) Skip in basic?
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

impl Default for StoredData {
    fn default() -> Self {
        StoredData {
            last_update_time: Instant::now(),
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

impl StoredData {
    pub fn reset(&mut self) {
        *self = StoredData::default();
    }

    #[allow(
        clippy::boxed_local,
        reason = "This avoids warnings on certain platforms (e.g. 32-bit)."
    )]
    fn eat_data(&mut self, data: Box<Data>, settings: &AppConfigFields) {
        let harvested_time = data.collection_time;

        self.timeseries_data.add(&data);

        if let Some(network) = data.network {
            self.network_harvest = network;
        }

        if let Some(memory) = data.memory {
            self.ram_harvest = memory;
        }

        self.swap_harvest = data.swap;

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

        if let Some(cpu) = data.cpu {
            self.cpu_harvest = cpu;
        }

        if let Some(load_avg) = data.load_avg {
            self.load_avg_harvest = load_avg;
        }

        // TODO: (points_rework_v1) the map might be redundant, the types are the same.
        self.temp_data = data
            .temperature_sensors
            .map(|sensors| {
                sensors
                    .into_iter()
                    .map(|temp| TempWidgetData {
                        sensor: temp.name,
                        temperature: temp
                            .temperature
                            .map(|c| settings.temperature_type.convert_temp_unit(c)),
                    })
                    .collect()
            })
            .unwrap_or_default();

        if let Some(disks) = data.disks {
            if let Some(io) = data.io {
                self.eat_disks(disks, io, harvested_time);
            }
        }

        if let Some(list_of_processes) = data.list_of_processes {
            self.process_data.ingest(list_of_processes);
        }

        #[cfg(feature = "battery")]
        {
            if let Some(list_of_batteries) = data.list_of_batteries {
                self.battery_harvest = list_of_batteries;
            }
        }

        // And we're done eating. Update time and push the new entry!
        self.last_update_time = harvested_time;
    }

    fn eat_disks(
        &mut self, disks: Vec<disks::DiskHarvest>, io: disks::IoHarvest, harvested_time: Instant,
    ) {
        let time_since_last_harvest = harvested_time
            .duration_since(self.last_update_time)
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
    Frozen(Box<StoredData>),
}

/// What data to share to other parts of the application.
#[derive(Default)]
pub struct DataStore {
    frozen_state: FrozenState,
    main: StoredData,
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

    /// Return a reference to the currently available data. Note that if the data is
    /// in a frozen state, it will return the snapshot of data from when it was frozen.
    pub fn get_data(&self) -> &StoredData {
        match &self.frozen_state {
            FrozenState::NotFrozen => &self.main,
            FrozenState::Frozen(collected_data) => collected_data,
        }
    }

    /// Eat data.
    pub fn eat_data(&mut self, data: Box<Data>, settings: &AppConfigFields) {
        self.main.eat_data(data, settings);
    }

    /// Clean data.
    pub fn clean_data(&mut self, max_duration: Duration) {
        self.main.timeseries_data.prune(max_duration);
    }

    /// Reset data state.
    pub fn reset(&mut self) {
        self.frozen_state = FrozenState::NotFrozen;
        self.main = StoredData::default();
    }
}
