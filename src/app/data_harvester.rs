//! This is the main file to house data collection functions.

use std::time::{Duration, Instant};

#[cfg(any(target_os = "linux", feature = "gpu"))]
use hashbrown::HashMap;
#[cfg(feature = "battery")]
use starship_battery::{Battery, Manager};
use sysinfo::{System, SystemExt};

use self::temperature::TemperatureType;
use super::DataFilters;
use crate::app::layout_manager::UsedWidgets;

#[cfg(feature = "nvidia")]
pub mod nvidia;

#[cfg(feature = "battery")]
pub mod batteries;

pub mod cpu;
pub mod disks;
pub mod memory;
pub mod network;
pub mod processes;
pub mod temperature;

#[derive(Clone, Debug)]
pub struct Data {
    pub collection_time: Instant,
    pub cpu: Option<cpu::CpuHarvest>,
    pub load_avg: Option<cpu::LoadAvgHarvest>,
    pub memory: Option<memory::MemHarvest>,
    #[cfg(not(target_os = "windows"))]
    pub cache: Option<memory::MemHarvest>,
    pub swap: Option<memory::MemHarvest>,
    pub temperature_sensors: Option<Vec<temperature::TempHarvest>>,
    pub network: Option<network::NetworkHarvest>,
    pub list_of_processes: Option<Vec<processes::ProcessHarvest>>,
    pub disks: Option<Vec<disks::DiskHarvest>>,
    pub io: Option<disks::IoHarvest>,
    #[cfg(feature = "battery")]
    pub list_of_batteries: Option<Vec<batteries::BatteryHarvest>>,
    #[cfg(feature = "zfs")]
    pub arc: Option<memory::MemHarvest>,
    #[cfg(feature = "gpu")]
    pub gpu: Option<Vec<(String, memory::MemHarvest)>>,
}

impl Default for Data {
    fn default() -> Self {
        Data {
            collection_time: Instant::now(),
            cpu: None,
            load_avg: None,
            memory: None,
            #[cfg(not(target_os = "windows"))]
            cache: None,
            swap: None,
            temperature_sensors: None,
            list_of_processes: None,
            disks: None,
            io: None,
            network: None,
            #[cfg(feature = "battery")]
            list_of_batteries: None,
            #[cfg(feature = "zfs")]
            arc: None,
            #[cfg(feature = "gpu")]
            gpu: None,
        }
    }
}

impl Data {
    pub fn cleanup(&mut self) {
        self.io = None;
        self.temperature_sensors = None;
        self.list_of_processes = None;
        self.disks = None;
        self.memory = None;
        self.swap = None;
        self.cpu = None;
        self.load_avg = None;

        if let Some(network) = &mut self.network {
            network.first_run_cleanup();
        }
        #[cfg(feature = "zfs")]
        {
            self.arc = None;
        }
        #[cfg(feature = "gpu")]
        {
            self.gpu = None;
        }
    }
}

#[derive(Debug)]
pub struct DataCollector {
    pub data: Data,
    sys: System,
    temperature_type: TemperatureType,
    use_current_cpu_total: bool,
    unnormalized_cpu: bool,
    last_collection_time: Instant,
    total_rx: u64,
    total_tx: u64,
    show_average_cpu: bool,
    widgets_to_harvest: UsedWidgets,
    filters: DataFilters,

    #[cfg(target_os = "linux")]
    pid_mapping: HashMap<crate::Pid, processes::PrevProcDetails>,
    #[cfg(target_os = "linux")]
    prev_idle: f64,
    #[cfg(target_os = "linux")]
    prev_non_idle: f64,

    #[cfg(feature = "battery")]
    battery_manager: Option<Manager>,
    #[cfg(feature = "battery")]
    battery_list: Option<Vec<Battery>>,

    #[cfg(target_family = "unix")]
    user_table: self::processes::UserTable,

    #[cfg(feature = "gpu")]
    gpu_pids: Option<Vec<HashMap<u32, (u64, u32)>>>,
}

impl DataCollector {
    pub fn new(filters: DataFilters) -> Self {
        DataCollector {
            data: Data::default(),
            sys: System::new_with_specifics(sysinfo::RefreshKind::new()),
            #[cfg(target_os = "linux")]
            pid_mapping: HashMap::default(),
            #[cfg(target_os = "linux")]
            prev_idle: 0_f64,
            #[cfg(target_os = "linux")]
            prev_non_idle: 0_f64,
            temperature_type: TemperatureType::Celsius,
            use_current_cpu_total: false,
            unnormalized_cpu: false,
            last_collection_time: Instant::now(),
            total_rx: 0,
            total_tx: 0,
            show_average_cpu: false,
            widgets_to_harvest: UsedWidgets::default(),
            #[cfg(feature = "battery")]
            battery_manager: None,
            #[cfg(feature = "battery")]
            battery_list: None,
            filters,
            #[cfg(target_family = "unix")]
            user_table: Default::default(),
            #[cfg(feature = "gpu")]
            gpu_pids: None,
        }
    }

    pub fn init(&mut self) {
        #[cfg(feature = "battery")]
        {
            if self.widgets_to_harvest.use_battery {
                if let Ok(battery_manager) = Manager::new() {
                    if let Ok(batteries) = battery_manager.batteries() {
                        let battery_list: Vec<Battery> = batteries.filter_map(Result::ok).collect();
                        if !battery_list.is_empty() {
                            self.battery_list = Some(battery_list);
                            self.battery_manager = Some(battery_manager);
                        }
                    }
                }
            }
        }

        // Sysinfo-related list refreshing.
        if self.widgets_to_harvest.use_net {
            self.sys.refresh_networks_list();
        }

        if self.widgets_to_harvest.use_temp {
            self.sys.refresh_components_list();
        }

        #[cfg(target_os = "windows")]
        {
            if self.widgets_to_harvest.use_proc {
                self.sys.refresh_users_list();
            }

            if self.widgets_to_harvest.use_disk {
                self.sys.refresh_disks_list();
            }
        }

        self.update_data();

        // Sleep a few seconds to avoid potentially weird data.
        const SLEEP: Duration = get_sleep_duration();

        std::thread::sleep(SLEEP);
        self.data.cleanup();
    }

    pub fn set_data_collection(&mut self, used_widgets: UsedWidgets) {
        self.widgets_to_harvest = used_widgets;
    }

    pub fn set_temperature_type(&mut self, temperature_type: TemperatureType) {
        self.temperature_type = temperature_type;
    }

    pub fn set_use_current_cpu_total(&mut self, use_current_cpu_total: bool) {
        self.use_current_cpu_total = use_current_cpu_total;
    }

    pub fn set_unnormalized_cpu(&mut self, unnormalized_cpu: bool) {
        self.unnormalized_cpu = unnormalized_cpu;
    }

    pub fn set_show_average_cpu(&mut self, show_average_cpu: bool) {
        self.show_average_cpu = show_average_cpu;
    }

    /// Refresh sysinfo data. We use sysinfo for the following data:
    /// - CPU usage
    /// - Memory usage
    /// - Network usage
    /// - Processes (non-Linux)
    /// - Disk (Windows)
    /// - Temperatures (non-Linux)
    fn refresh_sysinfo_data(&mut self) {
        // Refresh once every minute. If it's too frequent it can cause segfaults.
        const LIST_REFRESH_TIME: Duration = Duration::from_secs(60);
        let refresh_start = Instant::now();

        if self.widgets_to_harvest.use_cpu || self.widgets_to_harvest.use_proc {
            self.sys.refresh_cpu();
        }

        if self.widgets_to_harvest.use_mem || self.widgets_to_harvest.use_proc {
            self.sys.refresh_memory();
        }

        if self.widgets_to_harvest.use_net {
            if refresh_start.duration_since(self.last_collection_time) > LIST_REFRESH_TIME {
                self.sys.refresh_networks_list();
            }
            self.sys.refresh_networks();
        }

        // sysinfo is used on non-Linux systems for the following:
        // - Processes (users list as well for Windows)
        // - Disks (Windows only)
        // - Temperatures and temperature components list.
        #[cfg(not(target_os = "linux"))]
        {
            if self.widgets_to_harvest.use_proc {
                // For Windows, sysinfo also handles the users list.
                #[cfg(target_os = "windows")]
                if refresh_start.duration_since(self.last_collection_time) > LIST_REFRESH_TIME {
                    self.sys.refresh_users_list();
                }

                self.sys.refresh_processes();
            }

            if self.widgets_to_harvest.use_temp {
                if refresh_start.duration_since(self.last_collection_time) > LIST_REFRESH_TIME {
                    self.sys.refresh_components_list();
                }
                self.sys.refresh_components();
            }
        }

        #[cfg(target_os = "windows")]
        if self.widgets_to_harvest.use_disk {
            if refresh_start.duration_since(self.last_collection_time) > LIST_REFRESH_TIME {
                self.sys.refresh_disks_list();
            }
            self.sys.refresh_disks();
        }
    }

    pub fn update_data(&mut self) {
        self.refresh_sysinfo_data();

        self.data.collection_time = Instant::now();

        self.update_cpu_usage();
        self.update_memory_usage();
        self.update_temps();
        #[cfg(feature = "battery")]
        self.update_batteries();
        #[cfg(feature = "gpu")]
        self.update_gpus(); // update_gpus before procs for gpu_pids but after temp/batteries/cpu_usage for appending
        self.update_processes();
        self.update_network_usage();
        self.update_disks();

        // Update times for future reference.
        self.last_collection_time = self.data.collection_time;
    }
    #[cfg(feature = "gpu")]
    #[inline]
    fn update_gpus(&mut self) {
        if self.widgets_to_harvest.use_gpu {
            let use_temp = self.widgets_to_harvest.use_temp;
            let use_mem = self.widgets_to_harvest.use_mem;
            let use_proc = self.widgets_to_harvest.use_proc;
            let use_cpu = self.widgets_to_harvest.use_cpu;
            let use_battery = self.widgets_to_harvest.use_battery;
            #[cfg(feature = "nvidia")]
            if let Some(data) = nvidia::get_nvidia_vecs(
                &self.temperature_type,
                &self.filters.temp_filter,
                use_temp,
                use_mem,
                use_proc,
                use_cpu,
                use_battery,
            ) {
                if use_temp {
                    if let Some(mut temp) = data.temperature {
                        if let Some(ref mut sensors) = self.data.temperature_sensors {
                            sensors.append(&mut temp);
                        } else {
                            self.data.temperature_sensors = Some(temp);
                        }
                    }
                }
                if use_mem {
                    if let Some(mem) = data.memory {
                        self.data.gpu = Some(mem);
                    }
                }
                if use_proc {
                    if let Some(proc) = data.procs {
                        self.gpu_pids = Some(proc);
                    }
                }
                if use_cpu {
                    if let Some(mut cpu) = data.usage {
                        if let Some(ref mut cpus) = self.data.cpu {
                            cpus.append(&mut cpu);
                        } else {
                            self.data.cpu = Some(cpu);
                        }
                    }
                }
                #[cfg(all(feature = "battery", target_os = "linux"))]
                {
                    use crate::data_harvester::batteries::BatteryHarvest;
                    use starship_battery::State;
                    if use_battery {
                        if let Some(power) = data.battery {
                            let mut powers: Vec<BatteryHarvest> = power
                                .into_iter()
                                .map(|pwr| {
                                    BatteryHarvest {
                                        charge_percent: 100.0,
                                        secs_until_full: None,
                                        secs_until_empty: None,
                                        power_consumption_rate_watts: (pwr.1 / 1000) as f64, // convert milliwatts to watts
                                        health_percent: 100.0,
                                        state: State::Unknown,
                                        name: pwr.0,
                                    }
                                })
                                .collect();
                            if let Some(ref mut batts) = self.data.list_of_batteries {
                                batts.append(&mut powers);
                            } else {
                                self.data.list_of_batteries = Some(powers);
                            }
                        }
                    }
                }
            }
        }
    }

    #[inline]
    fn update_cpu_usage(&mut self) {
        if self.widgets_to_harvest.use_cpu {
            self.data.cpu = cpu::get_cpu_data_list(&self.sys, self.show_average_cpu).ok();

            #[cfg(target_family = "unix")]
            {
                self.data.load_avg = cpu::get_load_avg().ok();
            }
        }
    }

    #[inline]
    fn update_processes(&mut self) {
        if self.widgets_to_harvest.use_proc {
            if let Ok(mut process_list) = self.get_processes() {
                // NB: To avoid duplicate sorts on rerenders/events, we sort the processes by PID here.
                // We also want to avoid re-sorting *again* later on if we're sorting by PID, since we already
                // did it here!
                process_list.sort_unstable_by_key(|p| p.pid);
                self.data.list_of_processes = Some(process_list);
            }
        }
    }

    #[inline]
    fn update_temps(&mut self) {
        if self.widgets_to_harvest.use_temp {
            #[cfg(not(target_os = "linux"))]
            if let Ok(data) = temperature::get_temperature_data(
                &self.sys,
                &self.temperature_type,
                &self.filters.temp_filter,
            ) {
                self.data.temperature_sensors = data;
            }

            #[cfg(target_os = "linux")]
            if let Ok(data) =
                temperature::get_temperature_data(&self.temperature_type, &self.filters.temp_filter)
            {
                self.data.temperature_sensors = data;
            }
        }
    }

    #[inline]
    fn update_memory_usage(&mut self) {
        if self.widgets_to_harvest.use_mem {
            self.data.memory = memory::get_ram_usage(&self.sys);

            #[cfg(not(target_os = "windows"))]
            if self.widgets_to_harvest.use_cache {
                self.data.cache = memory::get_cache_usage(&self.sys);
            }

            self.data.swap = memory::get_swap_usage(
                #[cfg(not(target_os = "windows"))]
                &self.sys,
            );

            #[cfg(feature = "zfs")]
            {
                self.data.arc = memory::arc::get_arc_usage();
            }
        }
    }

    #[inline]
    fn update_network_usage(&mut self) {
        let current_instant = self.data.collection_time;

        if self.widgets_to_harvest.use_net {
            let net_data = network::get_network_data(
                &self.sys,
                self.last_collection_time,
                &mut self.total_rx,
                &mut self.total_tx,
                current_instant,
                &self.filters.net_filter,
            );

            self.total_rx = net_data.total_rx;
            self.total_tx = net_data.total_tx;
            self.data.network = Some(net_data);
        }
    }

    #[inline]
    #[cfg(feature = "battery")]
    fn update_batteries(&mut self) {
        if let Some(battery_manager) = &self.battery_manager {
            if let Some(battery_list) = &mut self.battery_list {
                self.data.list_of_batteries =
                    Some(batteries::refresh_batteries(battery_manager, battery_list));
            }
        }
    }

    #[inline]
    fn update_disks(&mut self) {
        if self.widgets_to_harvest.use_disk {
            self.data.disks = disks::get_disk_usage(self).ok();
            self.data.io = disks::get_io_usage().ok();
        }
    }

    /// Returns the total memory of the system.
    #[inline]
    fn total_memory(&self) -> u64 {
        if let Some(memory) = &self.data.memory {
            memory.total_bytes
        } else {
            self.sys.total_memory()
        }
    }
}

/// We set a sleep duration between 10ms and 250ms, ideally sysinfo's [`System::MINIMUM_CPU_UPDATE_INTERVAL`] + 1.
///
/// We bound the upper end to avoid waiting too long (e.g. FreeBSD is 1s, which I'm fine with losing
/// accuracy on for the first refresh), and we bound the lower end just to avoid the off-chance that
/// refreshing too quickly causes problems. This second case should only happen on unsupported
/// systems via sysinfo, in which case [`System::MINIMUM_CPU_UPDATE_INTERVAL`] is defined as 0.
///
/// We also do `INTERVAL + 1` for some wiggle room, just in case.
const fn get_sleep_duration() -> Duration {
    const MIN_SLEEP: u64 = 10;
    const MAX_SLEEP: u64 = 250;
    const INTERVAL: u64 = System::MINIMUM_CPU_UPDATE_INTERVAL.as_millis() as u64;

    if INTERVAL < MIN_SLEEP {
        Duration::from_millis(MIN_SLEEP)
    } else if INTERVAL > MAX_SLEEP {
        Duration::from_millis(MAX_SLEEP)
    } else {
        Duration::from_millis(INTERVAL + 1)
    }
}

#[cfg(target_os = "freebsd")]
/// Deserialize [libxo](https://www.freebsd.org/cgi/man.cgi?query=libxo&apropos=0&sektion=0&manpath=FreeBSD+13.1-RELEASE+and+Ports&arch=default&format=html) JSON data
fn deserialize_xo<T>(key: &str, data: &[u8]) -> Result<T, std::io::Error>
where
    T: serde::de::DeserializeOwned,
{
    let mut value: serde_json::Value = serde_json::from_slice(data)?;
    value
        .as_object_mut()
        .and_then(|map| map.remove(key))
        .ok_or_else(|| std::io::Error::new(std::io::ErrorKind::Other, "key not found"))
        .and_then(|val| serde_json::from_value(val).map_err(|err| err.into()))
}
