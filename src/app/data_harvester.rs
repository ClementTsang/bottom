//! This is the main file to house data collection functions.

use std::{collections::HashMap, time::Instant};

use sysinfo::{System, SystemExt};

use crate::app::layout_manager::UsedWidgets;

use futures::join;

pub mod cpu;
pub mod disks;
pub mod mem;
pub mod network;
pub mod processes;
pub mod temperature;

#[derive(Clone, Debug)]
pub struct Data {
    pub cpu: cpu::CPUHarvest,
    pub memory: mem::MemHarvest,
    pub swap: mem::MemHarvest,
    pub temperature_sensors: Vec<temperature::TempHarvest>,
    pub network: network::NetworkHarvest,
    pub list_of_processes: Vec<processes::ProcessHarvest>,
    pub disks: Vec<disks::DiskHarvest>,
    pub io: disks::IOHarvest,
    pub last_collection_time: Instant,
}

impl Default for Data {
    fn default() -> Self {
        Data {
            cpu: cpu::CPUHarvest::default(),
            memory: mem::MemHarvest::default(),
            swap: mem::MemHarvest::default(),
            temperature_sensors: Vec::default(),
            list_of_processes: Vec::default(),
            disks: Vec::default(),
            io: disks::IOHarvest::default(),
            network: network::NetworkHarvest::default(),
            last_collection_time: Instant::now(),
        }
    }
}

impl Data {
    pub fn first_run_cleanup(&mut self) {
        self.io = disks::IOHarvest::default();
        self.temperature_sensors = Vec::new();
        self.list_of_processes = Vec::new();
        self.disks = Vec::new();

        self.network.first_run_cleanup();
        self.memory = mem::MemHarvest::default();
        self.swap = mem::MemHarvest::default();
        self.cpu = cpu::CPUHarvest::default();
    }
}

pub struct DataCollector {
    pub data: Data,
    sys: System,
    prev_pid_stats: HashMap<u32, processes::PrevProcDetails>,
    prev_idle: f64,
    prev_non_idle: f64,
    mem_total_kb: u64,
    temperature_type: temperature::TemperatureType,
    use_current_cpu_total: bool,
    last_collection_time: Instant,
    total_rx: u64,
    total_tx: u64,
    show_average_cpu: bool,
    widgets_to_harvest: UsedWidgets,
}

impl Default for DataCollector {
    fn default() -> Self {
        DataCollector {
            data: Data::default(),
            sys: System::new_all(),
            prev_pid_stats: HashMap::new(),
            prev_idle: 0_f64,
            prev_non_idle: 0_f64,
            mem_total_kb: 0,
            temperature_type: temperature::TemperatureType::Celsius,
            use_current_cpu_total: false,
            last_collection_time: Instant::now(),
            total_rx: 0,
            total_tx: 0,
            show_average_cpu: false,
            widgets_to_harvest: UsedWidgets::default(),
        }
    }
}

impl DataCollector {
    pub fn init(&mut self) {
        self.mem_total_kb = self.sys.get_total_memory();
        futures::executor::block_on(self.update_data());
        std::thread::sleep(std::time::Duration::from_millis(250));
        self.data.first_run_cleanup();
    }

    pub fn set_collected_data(&mut self, used_widgets: UsedWidgets) {
        self.widgets_to_harvest = used_widgets;
    }

    pub fn set_temperature_type(&mut self, temperature_type: temperature::TemperatureType) {
        self.temperature_type = temperature_type;
    }

    pub fn set_use_current_cpu_total(&mut self, use_current_cpu_total: bool) {
        self.use_current_cpu_total = use_current_cpu_total;
    }

    pub fn set_show_average_cpu(&mut self, show_average_cpu: bool) {
        self.show_average_cpu = show_average_cpu;
    }

    pub async fn update_data(&mut self) {
        if self.widgets_to_harvest.use_cpu {
            self.sys.refresh_cpu();
        }

        if cfg!(not(target_os = "linux")) {
            if self.widgets_to_harvest.use_proc {
                self.sys.refresh_processes();
            }
            if self.widgets_to_harvest.use_temp {
                self.sys.refresh_components();
            }
        }
        if cfg!(target_os = "windows") && self.widgets_to_harvest.use_net {
            self.sys.refresh_networks();
        }

        let current_instant = std::time::Instant::now();

        // CPU
        if self.widgets_to_harvest.use_cpu {
            self.data.cpu = cpu::get_cpu_data_list(&self.sys, self.show_average_cpu);
        }

        if self.widgets_to_harvest.use_proc {
            // Processes.  This is the longest part of the harvesting process... changing this might be
            // good in the future.  What was tried already:
            // * Splitting the internal part into multiple scoped threads (dropped by ~.01 seconds, but upped usage)
            if let Ok(process_list) = if cfg!(target_os = "linux") {
                processes::linux_get_processes_list(
                    &mut self.prev_idle,
                    &mut self.prev_non_idle,
                    &mut self.prev_pid_stats,
                    self.use_current_cpu_total,
                    current_instant
                        .duration_since(self.last_collection_time)
                        .as_secs(),
                )
            } else {
                processes::windows_macos_get_processes_list(
                    &self.sys,
                    self.use_current_cpu_total,
                    self.mem_total_kb,
                )
            } {
                self.data.list_of_processes = process_list;
            }
        }

        // ASYNC
        let network_data_fut = network::get_network_data(
            &self.sys,
            self.last_collection_time,
            &mut self.total_rx,
            &mut self.total_tx,
            current_instant,
            self.widgets_to_harvest.use_net,
        );

        let mem_data_fut = mem::get_mem_data_list(self.widgets_to_harvest.use_mem);
        let swap_data_fut = mem::get_swap_data_list(self.widgets_to_harvest.use_mem);
        let disk_data_fut = disks::get_disk_usage_list(self.widgets_to_harvest.use_disk);
        let disk_io_usage_fut = disks::get_io_usage_list(false, self.widgets_to_harvest.use_disk);
        let temp_data_fut = temperature::get_temperature_data(
            &self.sys,
            &self.temperature_type,
            self.widgets_to_harvest.use_temp,
        );

        let (net_data, mem_res, swap_res, disk_res, io_res, temp_res) = join!(
            network_data_fut,
            mem_data_fut,
            swap_data_fut,
            disk_data_fut,
            disk_io_usage_fut,
            temp_data_fut
        );

        // After async
        if let Some(net_data) = net_data {
            self.data.network = net_data;
            self.total_rx = self.data.network.total_rx;
            self.total_tx = self.data.network.total_tx;
        }

        if let Ok(memory) = mem_res {
            if let Some(memory) = memory {
                self.data.memory = memory;
            }
        }

        if let Ok(swap) = swap_res {
            if let Some(swap) = swap {
                self.data.swap = swap;
            }
        }

        if let Ok(disks) = disk_res {
            if let Some(disks) = disks {
                self.data.disks = disks;
            }
        }

        if let Ok(io) = io_res {
            if let Some(io) = io {
                self.data.io = io;
            }
        }

        if let Ok(temp) = temp_res {
            if let Some(temp) = temp {
                self.data.temperature_sensors = temp;
            }
        }

        // Update time
        self.data.last_collection_time = current_instant;
        self.last_collection_time = current_instant;
    }
}
