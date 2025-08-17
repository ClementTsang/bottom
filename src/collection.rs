//! This is the main file to house data collection functions.
//!
//! TODO: Rename this to intake? Collection?

#[cfg(feature = "nvidia")]
pub mod nvidia;

#[cfg(all(target_os = "linux", feature = "gpu"))]
pub mod amd;

#[cfg(target_os = "linux")]
mod linux {
    pub mod utils;
}

#[cfg(feature = "battery")]
pub mod batteries;
pub mod cpu;
pub mod disks;
pub mod error;
pub mod memory;
pub mod network;
pub mod processes;
pub mod temperature;

use std::time::{Duration, Instant};

#[cfg(any(target_os = "linux", feature = "gpu"))]
use hashbrown::HashMap;
#[cfg(not(target_os = "windows"))]
use processes::Pid;
#[cfg(feature = "battery")]
use starship_battery::{Battery, Manager};

use super::DataFilters;
use crate::app::layout_manager::UsedWidgets;

// TODO: We can possibly re-use an internal buffer for this to reduce allocs.
#[derive(Clone, Debug)]
pub struct Data {
    pub collection_time: Instant,
    pub cpu: Option<cpu::CpuHarvest>,
    pub load_avg: Option<cpu::LoadAvgHarvest>,
    pub memory: Option<memory::MemData>,
    #[cfg(not(target_os = "windows"))]
    pub cache: Option<memory::MemData>,
    pub swap: Option<memory::MemData>,
    pub temperature_sensors: Option<Vec<temperature::TempSensorData>>,
    pub network: Option<network::NetworkHarvest>,
    pub list_of_processes: Option<Vec<processes::ProcessHarvest>>,
    pub disks: Option<Vec<disks::DiskHarvest>>,
    pub io: Option<disks::IoHarvest>,
    #[cfg(feature = "battery")]
    pub list_of_batteries: Option<Vec<batteries::BatteryData>>,
    #[cfg(feature = "zfs")]
    pub arc: Option<memory::MemData>,
    #[cfg(feature = "gpu")]
    pub gpu: Option<Vec<(String, memory::MemData)>>,
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

/// A wrapper around the sysinfo data source. We use sysinfo for the following
/// data:
/// - CPU usage
/// - Memory usage
/// - Network usage
/// - Processes (non-Linux)
/// - Disk (anything outside of Linux, macOS, and FreeBSD)
/// - Temperatures (non-Linux)
#[derive(Debug)]
pub struct SysinfoSource {
    /// Handles CPU, memory, and processes.
    pub(crate) system: sysinfo::System,
    pub(crate) network: sysinfo::Networks,
    #[cfg(not(target_os = "linux"))]
    pub(crate) temps: sysinfo::Components,
    #[cfg(not(any(target_os = "linux", target_os = "macos", target_os = "freebsd")))]
    pub(crate) disks: sysinfo::Disks,
    #[cfg(target_os = "windows")]
    pub(crate) users: sysinfo::Users,
}

impl Default for SysinfoSource {
    fn default() -> Self {
        use sysinfo::*;

        Self {
            system: System::new(),
            network: Networks::new(),
            #[cfg(not(target_os = "linux"))]
            temps: Components::new(),
            #[cfg(not(any(target_os = "linux", target_os = "macos", target_os = "freebsd")))]
            disks: Disks::new(),
            #[cfg(target_os = "windows")]
            users: Users::new(),
        }
    }
}

#[derive(Debug)]
pub struct DataCollector {
    pub data: Data,
    sys: SysinfoSource,
    last_collection_time: Instant,
    widgets_to_harvest: UsedWidgets,
    filters: DataFilters,

    total_rx: u64,
    total_tx: u64,

    unnormalized_cpu: bool,
    use_current_cpu_total: bool,
    show_average_cpu: bool,
    get_process_threads: bool,

    last_list_collection_time: Instant,
    should_run_less_routine_tasks: bool,

    #[cfg(target_os = "linux")]
    prev_process_details: HashMap<Pid, processes::PrevProcDetails>,
    #[cfg(target_os = "linux")]
    prev_idle: f64,
    #[cfg(target_os = "linux")]
    prev_non_idle: f64,

    #[cfg(feature = "battery")]
    battery_manager: Option<Manager>,
    #[cfg(feature = "battery")]
    battery_list: Option<Vec<Battery>>,

    #[cfg(target_family = "unix")]
    user_table: processes::UserTable,

    #[cfg(feature = "gpu")]
    gpu_pids: Option<Vec<HashMap<u32, (u64, u32)>>>,
    #[cfg(feature = "gpu")]
    gpus_total_mem: Option<u64>,
}

const LESS_ROUTINE_TASK_TIME: Duration = Duration::from_secs(60);

impl DataCollector {
    pub fn new(filters: DataFilters) -> Self {
        // Initialize it to the past to force it to load on initialization.
        let now = Instant::now();
        let last_collection_time = now.checked_sub(LESS_ROUTINE_TASK_TIME * 10).unwrap_or(now);

        DataCollector {
            data: Data::default(),
            sys: SysinfoSource::default(),
            #[cfg(target_os = "linux")]
            prev_process_details: HashMap::default(),
            #[cfg(target_os = "linux")]
            prev_idle: 0_f64,
            #[cfg(target_os = "linux")]
            prev_non_idle: 0_f64,
            use_current_cpu_total: false,
            unnormalized_cpu: false,
            get_process_threads: false,
            last_collection_time,
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
            #[cfg(feature = "gpu")]
            gpus_total_mem: None,
            last_list_collection_time: last_collection_time,
            should_run_less_routine_tasks: true,
        }
    }

    /// Update the check for routine tasks like updating lists of batteries, cleanup, etc.
    /// This is useful for things that we don't want to update all the time.
    ///
    /// Note this should be set back to false if `self.last_list_collection_time` is updated.
    #[inline]
    fn run_less_routine_tasks(&mut self) {
        if self
            .data
            .collection_time
            .duration_since(self.last_list_collection_time)
            > LESS_ROUTINE_TASK_TIME
        {
            self.should_run_less_routine_tasks = true;
        }

        if self.should_run_less_routine_tasks {
            self.last_list_collection_time = self.data.collection_time;
        }
    }

    pub fn set_collection(&mut self, used_widgets: UsedWidgets) {
        self.widgets_to_harvest = used_widgets;
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

    pub fn set_get_process_threads(&mut self, get_process_threads: bool) {
        self.get_process_threads = get_process_threads;
    }

    /// Refresh sysinfo data. We use sysinfo for the following data:
    /// - CPU usage
    /// - Memory usage
    /// - Network usage
    /// - Processes (non-Linux)
    /// - Disk (Windows)
    /// - Temperatures (non-Linux)
    fn refresh_sysinfo_data(&mut self) {
        // Refresh the list of objects once every minute. If it's too frequent it can
        // cause segfaults.

        if self.widgets_to_harvest.use_cpu || self.widgets_to_harvest.use_proc {
            self.sys.system.refresh_cpu_all();
        }

        if self.widgets_to_harvest.use_mem || self.widgets_to_harvest.use_proc {
            self.sys.system.refresh_memory();
        }

        if self.widgets_to_harvest.use_net {
            self.sys.network.refresh(true);
        }

        // sysinfo is used on non-Linux systems for the following:
        // - Processes (users list as well for Windows)
        // - Disks (Windows only)
        // - Temperatures and temperature components list.
        #[cfg(not(target_os = "linux"))]
        {
            if self.widgets_to_harvest.use_proc {
                self.sys.system.refresh_processes_specifics(
                    sysinfo::ProcessesToUpdate::All,
                    true,
                    sysinfo::ProcessRefreshKind::everything()
                        .without_environ()
                        .without_cwd()
                        .without_root(),
                );

                // For Windows, sysinfo also handles the users list.
                #[cfg(target_os = "windows")]
                if self.should_run_less_routine_tasks {
                    self.sys.users.refresh();
                }
            }

            if self.widgets_to_harvest.use_temp {
                if self.should_run_less_routine_tasks {
                    self.sys.temps.refresh(true);
                }

                for component in self.sys.temps.iter_mut() {
                    component.refresh();
                }
            }

            #[cfg(target_os = "windows")]
            if self.widgets_to_harvest.use_disk {
                if self.should_run_less_routine_tasks {
                    self.sys.disks.refresh(true);
                }

                for disk in self.sys.disks.iter_mut() {
                    disk.refresh();
                }
            }
        }
    }

    /// Update and refresh data.
    ///
    /// TODO: separate refresh steps and update steps
    pub fn update_data(&mut self) {
        self.data.collection_time = Instant::now();

        self.run_less_routine_tasks();

        self.refresh_sysinfo_data();

        self.update_cpu_usage();
        self.update_memory_usage();
        self.update_temps();

        #[cfg(feature = "battery")]
        self.update_batteries();

        #[cfg(feature = "gpu")]
        self.update_gpus();

        self.update_processes();
        self.update_network_usage();
        self.update_disks();

        // Make sure to run this to refresh the setting.
        self.should_run_less_routine_tasks = false;

        // Update times for future reference.
        self.last_collection_time = self.data.collection_time;
    }

    /// Gets GPU data. Note this will usually append to other previously
    /// collected data fields at the moment.
    #[cfg(feature = "gpu")]
    #[inline]
    fn update_gpus(&mut self) {
        if self.widgets_to_harvest.use_gpu {
            let mut local_gpu: Vec<(String, memory::MemData)> = Vec::new();
            let mut local_gpu_pids: Vec<HashMap<u32, (u64, u32)>> = Vec::new();
            let mut local_gpu_total_mem: u64 = 0;

            #[cfg(feature = "nvidia")]
            if let Some(data) =
                nvidia::get_nvidia_vecs(&self.filters.temp_filter, &self.widgets_to_harvest)
            {
                if let Some(mut temp) = data.temperature {
                    if let Some(sensors) = &mut self.data.temperature_sensors {
                        sensors.append(&mut temp);
                    } else {
                        self.data.temperature_sensors = Some(temp);
                    }
                }
                if let Some(mut mem) = data.memory {
                    local_gpu.append(&mut mem);
                }
                if let Some(mut proc) = data.procs {
                    local_gpu_pids.append(&mut proc.1);
                    local_gpu_total_mem += proc.0;
                }
            }

            #[cfg(target_os = "linux")]
            if let Some(data) =
                amd::get_amd_vecs(&self.widgets_to_harvest, self.last_collection_time)
            {
                if let Some(mut mem) = data.memory {
                    local_gpu.append(&mut mem);
                }
                if let Some(mut proc) = data.procs {
                    local_gpu_pids.append(&mut proc.1);
                    local_gpu_total_mem += proc.0;
                }
            }

            self.data.gpu = (!local_gpu.is_empty()).then_some(local_gpu);
            self.gpu_pids = (!local_gpu_pids.is_empty()).then_some(local_gpu_pids);
            self.gpus_total_mem = (local_gpu_total_mem > 0).then_some(local_gpu_total_mem);
        }
    }

    #[inline]
    fn update_cpu_usage(&mut self) {
        if self.widgets_to_harvest.use_cpu {
            self.data.cpu = cpu::get_cpu_data_list(&self.sys.system, self.show_average_cpu).ok();

            #[cfg(target_family = "unix")]
            {
                self.data.load_avg = Some(cpu::get_load_avg());
            }
        }
    }

    #[inline]
    fn update_processes(&mut self) {
        if self.widgets_to_harvest.use_proc {
            if let Ok(mut process_list) = self.get_processes() {
                // NB: To avoid duplicate sorts on rerenders/events, we sort the processes by
                // PID here. We also want to avoid re-sorting *again* later on
                // if we're sorting by PID, since we already did it here!
                process_list.sort_unstable_by_key(|p| p.pid);
                self.data.list_of_processes = Some(process_list);
            }
        }
    }

    #[inline]
    fn update_temps(&mut self) {
        if self.widgets_to_harvest.use_temp {
            #[cfg(not(target_os = "linux"))]
            if let Ok(data) =
                temperature::get_temperature_data(&self.sys.temps, &self.filters.temp_filter)
            {
                self.data.temperature_sensors = data;
            }

            #[cfg(target_os = "linux")]
            if let Ok(data) = temperature::get_temperature_data(&self.filters.temp_filter) {
                self.data.temperature_sensors = data;
            }
        }
    }

    #[inline]
    fn update_memory_usage(&mut self) {
        if self.widgets_to_harvest.use_mem {
            self.data.memory = memory::get_ram_usage(&self.sys.system);

            #[cfg(not(target_os = "windows"))]
            if self.widgets_to_harvest.use_cache {
                self.data.cache = memory::get_cache_usage(&self.sys.system);
            }

            self.data.swap = memory::get_swap_usage(&self.sys.system);

            #[cfg(feature = "zfs")]
            {
                self.data.arc = memory::arc::get_arc_usage();
            }
        }
    }

    #[inline]
    fn update_network_usage(&mut self) {
        if self.widgets_to_harvest.use_net {
            let net_data = network::get_network_data(
                &self.sys.network,
                self.last_collection_time,
                &mut self.total_rx,
                &mut self.total_tx,
                self.data.collection_time,
                &self.filters.net_filter,
            );

            self.total_rx = net_data.total_rx;
            self.total_tx = net_data.total_tx;
            self.data.network = Some(net_data);
        }
    }

    /// Update battery information.
    ///
    /// If the battery manager is not initialized, it will attempt to initialize it if at least one battery is found.
    ///
    /// This function also refreshes the list of batteries if `self.should_run_less_routine_tasks` is true.
    #[inline]
    #[cfg(feature = "battery")]
    fn update_batteries(&mut self) {
        let battery_manager = match &self.battery_manager {
            Some(manager) => {
                // Also check if we need to refresh the list of batteries.
                if self.should_run_less_routine_tasks {
                    let battery_list = manager
                        .batteries()
                        .map(|batteries| batteries.filter_map(Result::ok).collect::<Vec<_>>());

                    if let Ok(battery_list) = battery_list {
                        if battery_list.is_empty() {
                            self.battery_list = None;
                        } else {
                            self.battery_list = Some(battery_list);
                        }
                    } else {
                        self.battery_list = None;
                    }
                }

                manager
            }
            None => {
                if let Ok(manager) = Manager::new() {
                    let Ok(batteries) = manager.batteries() else {
                        return;
                    };

                    let battery_list = batteries.filter_map(Result::ok).collect::<Vec<_>>();

                    if battery_list.is_empty() {
                        return;
                    }

                    self.battery_list = Some(battery_list);
                    self.battery_manager.insert(manager)
                } else {
                    return;
                }
            }
        };

        self.data.list_of_batteries = self
            .battery_list
            .as_mut()
            .map(|battery_list| batteries::refresh_batteries(battery_manager, battery_list));
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
            memory.total_bytes.get()
        } else {
            self.sys.system.total_memory()
        }
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
        .ok_or_else(|| std::io::Error::other("key not found"))
        .and_then(|val| serde_json::from_value(val).map_err(|err| err.into()))
}
