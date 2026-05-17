use std::{
    time::{Duration, Instant},
    vec::Vec,
};

use super::{ProcessData, TimeSeriesData};
#[cfg(feature = "battery")]
use crate::collection::batteries;
use crate::{
    app::{AppConfigFields, DataFilters, filter::Filter, layout_manager::UsedWidgets},
    collection::{
        Data,
        cpu::{CpuHarvest, LoadAvgHarvest},
        disks,
        memory::MemData,
        network::NetworkHarvest,
    },
    utils::data_units::DataUnit,
    widgets::{DiskWidgetData, TempWidgetData},
};

/// A collection of data. This is where we dump data into.
///
/// TODO: Maybe reduce visibility of internal data, make it only accessible
/// through DataStore?
#[derive(Debug, Clone)]
pub struct StoredData {
    // FIXME: (points_rework_v1) we could be able to remove this with some more refactoring.
    pub last_update_time: Instant,
    pub time_series_data: TimeSeriesData,
    pub network_harvest: NetworkHarvest,
    pub ram_harvest: Option<MemData>,
    pub swap_harvest: Option<MemData>,
    #[cfg(not(target_os = "windows"))]
    pub cache_harvest: Option<MemData>,
    #[cfg(feature = "zfs")]
    pub arc_harvest: Option<MemData>,
    #[cfg(feature = "gpu")]
    pub gpu_harvest: Vec<(String, MemData)>,
    pub cpu_harvest: CpuHarvest,
    pub load_avg_harvest: LoadAvgHarvest,
    pub process_data: ProcessData,
    /// TODO: (points_rework_v1) Might be a better way to do this without having
    /// to store here?
    pub prev_io: Vec<(u64, u64)>,
    pub disk_harvest: Vec<DiskWidgetData>,
    pub temp_data: Vec<TempWidgetData>,
    #[cfg(feature = "battery")]
    pub battery_harvest: Vec<batteries::BatteryData>,
}

impl Default for StoredData {
    fn default() -> Self {
        StoredData {
            last_update_time: Instant::now(),
            time_series_data: TimeSeriesData::default(),
            network_harvest: NetworkHarvest::default(),
            ram_harvest: None,
            #[cfg(not(target_os = "windows"))]
            cache_harvest: None,
            swap_harvest: None,
            cpu_harvest: CpuHarvest::default(),
            load_avg_harvest: LoadAvgHarvest::default(),
            process_data: Default::default(),
            prev_io: Vec::default(),
            disk_harvest: Vec::default(),
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
    fn eat_data(
        &mut self, mut data: Box<Data>, settings: &AppConfigFields, used_widgets: &UsedWidgets,
        filters: &DataFilters,
    ) {
        let harvested_time = data.collection_time;

        // We must adjust all the network values to their selected type (defaults to
        // bits).
        if matches!(settings.network_unit_type, DataUnit::Byte) {
            if let Some(network) = &mut data.network {
                network.rx /= 8;
                network.tx /= 8;
            }
        }

        if !settings.use_basic_mode {
            self.time_series_data
                .add(&data, used_widgets, settings, filters);
        }

        if let Some(network) = data.network {
            self.network_harvest = network;
        }

        self.ram_harvest = data.memory;
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

        self.temp_data = data
            .temperature_sensors
            .map(|sensors| {
                sensors
                    .into_iter()
                    .filter(|temp| Filter::optional_should_keep(&filters.temp_filter, &temp.name))
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

        self.disk_harvest.clear();

        let prev_io_diff = disks.len().saturating_sub(self.prev_io.len());
        self.prev_io.reserve(prev_io_diff);
        self.prev_io.extend((0..prev_io_diff).map(|_| (0, 0)));

        for (itx, device) in disks.into_iter().enumerate() {
            let Some(checked_name) = ({
                #[cfg(target_os = "windows")]
                {
                    match &device.volume_name {
                        Some(volume_name) => Some(volume_name.as_str()),
                        None => device.name.split('/').next_back(),
                    }
                }
                #[cfg(not(target_os = "windows"))]
                {
                    #[cfg(any(feature = "zfs", target_os = "freebsd"))]
                    {
                        #[cfg(target_os = "freebsd")]
                        {
                            Some(device.name.as_str()) // sysinfo name is the mount_point
                        }
                        #[cfg(not(target_os = "freebsd"))]
                        {
                            if !device.name.starts_with('/') {
                                Some(device.name.as_str()) // use the whole name
                            } else {
                                device.name.split('/').next_back() // use device name
                            }
                        }
                    }
                    #[cfg(not(any(feature = "zfs", target_os = "freebsd")))]
                    {
                        device.name.split('/').next_back()
                    }
                }
            }) else {
                continue;
            };

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
                        .get_or_init(|| Regex::new(r"disk\d+").expect("valid regex"))
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

            let (mut io_read_rate_bytes, mut io_write_rate_bytes) = (None, None);
            if let Some(Some(io_device)) = io_device {
                if let Some(prev_io) = self.prev_io.get_mut(itx) {
                    io_read_rate_bytes = Some(
                        ((io_device.read_bytes.saturating_sub(prev_io.0)) as f64
                            / time_since_last_harvest)
                            .round() as u64,
                    );

                    io_write_rate_bytes = Some(
                        ((io_device.write_bytes.saturating_sub(prev_io.1)) as f64
                            / time_since_last_harvest)
                            .round() as u64,
                    );

                    *prev_io = (io_device.read_bytes, io_device.write_bytes);
                }
            }

            let summed_total_bytes = match (device.used_space, device.free_space) {
                (Some(used), Some(free)) => Some(used + free),
                _ => None,
            };

            self.disk_harvest.push(DiskWidgetData {
                name: device.name,
                mount_point: device.mount_point,
                free_bytes: device.free_space,
                used_bytes: device.used_space,
                total_bytes: device.total_space,
                summed_total_bytes,
                io_read_rate_bytes,
                io_write_rate_bytes,
            });
        }
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
pub struct DataStore {
    frozen_state: FrozenState,
    main: StoredData,
    used_widgets: UsedWidgets,
    filters: DataFilters,
}

impl DataStore {
    /// Create a new [`DataStore`]
    pub fn new(used_widgets: UsedWidgets) -> Self {
        Self {
            frozen_state: FrozenState::default(),
            main: StoredData::default(),
            used_widgets,
            filters: DataFilters::default(),
        }
    }

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

    /// Return a reference to the currently available data. Note that if the
    /// data is in a frozen state, it will return the snapshot of data from
    /// when it was frozen.
    pub fn get_data(&self) -> &StoredData {
        match &self.frozen_state {
            FrozenState::NotFrozen => &self.main,
            FrozenState::Frozen(collected_data) => collected_data,
        }
    }

    pub fn set_filters(&mut self, filters: DataFilters) {
        self.filters = filters;
    }

    /// Eat data.
    pub fn eat_data(&mut self, data: Box<Data>, settings: &AppConfigFields) {
        self.main
            .eat_data(data, settings, &self.used_widgets, &self.filters);
    }

    /// Clean data.
    pub fn clean_data(&mut self, max_duration: Duration) {
        self.main.time_series_data.prune(max_duration);
    }

    /// Reset data state.
    pub fn reset(&mut self) {
        self.frozen_state = FrozenState::NotFrozen;
        self.main = StoredData::default();
    }
}
