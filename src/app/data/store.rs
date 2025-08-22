use std::{
    time::{Duration, Instant},
    vec::Vec,
};

use super::{ProcessData, TimeSeriesData};
#[cfg(feature = "battery")]
use crate::collection::batteries;
use crate::{
    app::AppConfigFields,
    collection::{Data, cpu, disks, memory::MemData, network},
    dec_bytes_per_second_string,
    utils::data_units::DataUnit,
    widgets::{DiskWidgetData, TempWidgetData},
};

/// A collection of data. This is where we dump data into.
///
/// TODO: Maybe reduce visibility of internal data, make it only accessible through DataStore?
#[derive(Debug, Clone)]
pub struct StoredData {
    pub last_update_time: Instant, // FIXME: (points_rework_v1) we could be able to remove this with some more refactoring.
    pub timeseries_data: TimeSeriesData,
    pub network_harvest: network::NetworkHarvest,
    pub ram_harvest: Option<MemData>,
    pub swap_harvest: Option<MemData>,
    #[cfg(not(target_os = "windows"))]
    pub cache_harvest: Option<MemData>,
    #[cfg(feature = "zfs")]
    pub arc_harvest: Option<MemData>,
    #[cfg(feature = "gpu")]
    pub gpu_harvest: Vec<(String, MemData)>,
    pub cpu_harvest: Vec<cpu::CpuData>,
    pub load_avg_harvest: cpu::LoadAvgHarvest,
    pub process_data: ProcessData,
    /// TODO: (points_rework_v1) Might be a better way to do this without having to store here?
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
            timeseries_data: TimeSeriesData::default(),
            network_harvest: network::NetworkHarvest::default(),
            ram_harvest: None,
            #[cfg(not(target_os = "windows"))]
            cache_harvest: None,
            swap_harvest: None,
            cpu_harvest: Default::default(),
            load_avg_harvest: cpu::LoadAvgHarvest::default(),
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
    fn eat_data(&mut self, mut data: Box<Data>, settings: &AppConfigFields) {
        let harvested_time = data.collection_time;

        // We must adjust all the network values to their selected type (defaults to bits).
        if matches!(settings.network_unit_type, DataUnit::Byte) {
            if let Some(network) = &mut data.network {
                network.rx /= 8;
                network.tx /= 8;
            }
        }

        if !settings.use_basic_mode {
            self.timeseries_data.add(&data);
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
            self.cpu_harvest.clear();

            if let Some(avg) = cpu.avg {
                self.cpu_harvest.push(cpu::CpuData {
                    data_type: cpu::CpuDataType::Avg,
                    usage: avg,
                });
            }

            self.cpu_harvest
                .extend(
                    cpu.cpus
                        .into_iter()
                        .enumerate()
                        .map(|(core, usage)| cpu::CpuData {
                            data_type: cpu::CpuDataType::Cpu(core as u32),
                            usage,
                        }),
                );
        }

        if let Some(load_avg) = data.load_avg {
            self.load_avg_harvest = load_avg;
        }

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
                    #[cfg(feature = "zfs")]
                    {
                        if !device.name.starts_with('/') {
                            Some(device.name.as_str()) // use the whole zfs
                        // dataset name
                        } else {
                            device.name.split('/').next_back()
                        }
                    }
                    #[cfg(not(feature = "zfs"))]
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

            let (mut io_read, mut io_write) = ("N/A".into(), "N/A".into());
            if let Some(Some(io_device)) = io_device {
                if let Some(prev_io) = self.prev_io.get_mut(itx) {
                    let r_rate = ((io_device.read_bytes.saturating_sub(prev_io.0)) as f64
                        / time_since_last_harvest)
                        .round() as u64;

                    let w_rate = ((io_device.write_bytes.saturating_sub(prev_io.1)) as f64
                        / time_since_last_harvest)
                        .round() as u64;

                    *prev_io = (io_device.read_bytes, io_device.write_bytes);

                    io_read = dec_bytes_per_second_string(r_rate).into();
                    io_write = dec_bytes_per_second_string(w_rate).into();
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
                io_read,
                io_write,
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
