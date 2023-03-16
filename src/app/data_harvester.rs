//! This is the main file to house data collection functions.

use std::time::{Duration, Instant};

#[cfg(target_os = "linux")]
use fxhash::FxHashMap;

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
    pub last_collection_time: Instant,
    pub cpu: Option<cpu::CpuHarvest>,
    pub load_avg: Option<cpu::LoadAvgHarvest>,
    pub memory: Option<memory::MemHarvest>,
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
            last_collection_time: Instant::now(),
            cpu: None,
            load_avg: None,
            memory: None,
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
    pid_mapping: FxHashMap<crate::Pid, processes::PrevProcDetails>,
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
}

impl DataCollector {
    pub fn new(filters: DataFilters) -> Self {
        DataCollector {
            data: Data::default(),
            sys: System::new_with_specifics(sysinfo::RefreshKind::new()),
            #[cfg(target_os = "linux")]
            pid_mapping: FxHashMap::default(),
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

        if self.widgets_to_harvest.use_net {
            self.sys.refresh_networks_list();
        }
        if self.widgets_to_harvest.use_temp {
            self.sys.refresh_components_list();
        }
        #[cfg(target_os = "windows")]
        if self.widgets_to_harvest.use_proc {
            self.sys.refresh_users_list();
        }

        futures::executor::block_on(self.update_data());

        std::thread::sleep(std::time::Duration::from_millis(250));
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

    /// Refresh sysinfo data.
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

        #[cfg(not(target_os = "linux"))]
        {
            if self.widgets_to_harvest.use_proc {
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
    }

    pub async fn update_data(&mut self) {
        self.refresh_sysinfo_data();

        let current_instant = Instant::now();

        self.update_cpu_usage();
        self.update_memory_usage();
        self.update_processes(
            #[cfg(target_os = "linux")]
            current_instant,
        );
        self.update_temps();
        self.update_network_usage(current_instant);
        self.update_disks();

        #[cfg(feature = "battery")]
        self.update_batteries();

        // Update times for future reference.
        self.last_collection_time = current_instant;
        self.data.last_collection_time = current_instant;
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
    fn update_processes(&mut self, #[cfg(target_os = "linux")] current_instant: Instant) {
        if self.widgets_to_harvest.use_proc {
            if let Ok(mut process_list) = {
                let total_memory = if let Some(memory) = &self.data.memory {
                    memory.total_bytes
                } else {
                    self.sys.total_memory()
                };

                #[cfg(target_os = "linux")]
                {
                    use self::processes::{PrevProc, ProcHarvestOptions};

                    let prev_proc = PrevProc {
                        prev_idle: &mut self.prev_idle,
                        prev_non_idle: &mut self.prev_non_idle,
                    };

                    let proc_harvest_options = ProcHarvestOptions {
                        use_current_cpu_total: self.use_current_cpu_total,
                        unnormalized_cpu: self.unnormalized_cpu,
                    };

                    let time_diff = current_instant
                        .duration_since(self.last_collection_time)
                        .as_secs();

                    processes::get_process_data(
                        &self.sys,
                        prev_proc,
                        &mut self.pid_mapping,
                        proc_harvest_options,
                        time_diff,
                        total_memory,
                        &mut self.user_table,
                    )
                }
                #[cfg(not(target_os = "linux"))]
                {
                    #[cfg(target_family = "unix")]
                    {
                        processes::get_process_data(
                            &self.sys,
                            self.use_current_cpu_total,
                            self.unnormalized_cpu,
                            total_memory,
                            &mut self.user_table,
                        )
                    }
                    #[cfg(not(target_family = "unix"))]
                    {
                        processes::get_process_data(
                            &self.sys,
                            self.use_current_cpu_total,
                            self.unnormalized_cpu,
                            total_memory,
                        )
                    }
                }
            } {
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
            self.data.swap = memory::get_swap_usage(
                #[cfg(not(target_os = "windows"))]
                &self.sys,
            );

            #[cfg(feature = "zfs")]
            {
                self.data.arc = memory::arc::get_arc_usage();
            }

            #[cfg(feature = "gpu")]
            if self.widgets_to_harvest.use_gpu {
                self.data.gpu = memory::gpu::get_gpu_mem_usage();
            }
        }
    }

    #[inline]
    fn update_network_usage(&mut self, current_instant: Instant) {
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
            #[cfg(any(target_os = "freebsd", target_os = "linux", target_os = "macos"))]
            {
                self.data.disks =
                    disks::get_disk_usage(&self.filters.disk_filter, &self.filters.mount_filter)
                        .ok();
            }

            #[cfg(target_os = "windows")]
            {
                self.data.disks = Some(disks::get_disk_usage(
                    &self.sys,
                    &self.filters.disk_filter,
                    &self.filters.mount_filter,
                ));
            }

            self.data.io = disks::get_io_usage().ok();
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
        .ok_or_else(|| std::io::Error::new(std::io::ErrorKind::Other, "key not found"))
        .and_then(|val| serde_json::from_value(val).map_err(|err| err.into()))
}
