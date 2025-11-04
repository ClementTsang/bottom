// src/opentelemetry/adapter.rs
use super::config::ProcessFilterConfig;
use super::exporter::{
    CpuMetric, DiskMetric, GpuMetric, MemoryMetric, NetworkMetric, ProcessMetric, SystemMetrics,
    TemperatureMetric,
};

use crate::app::data::StoredData;

pub struct BottomDataAdapter {
    cpu_data: Vec<CpuMetric>,
    memory_data: Option<MemoryMetric>,
    network_data: Vec<NetworkMetric>,
    disk_data: Vec<DiskMetric>,
    process_data: Vec<ProcessMetric>,
    temperature_data: Vec<TemperatureMetric>,
    gpu_data: Vec<GpuMetric>,
}

impl BottomDataAdapter {
    /// Converti dai dati reali di bottom (clonando)
    pub fn from_stored_data(stored_data: &StoredData) -> Self {
        Self::from_stored_data_with_filter(stored_data, None)
    }

    /// Converti dai dati reali di bottom con filtro processi opzionale
    pub fn from_stored_data_with_filter(
        stored_data: &StoredData, process_filter: Option<&ProcessFilterConfig>,
    ) -> Self {
        // CPU
        let cpu_data: Vec<CpuMetric> = stored_data
            .cpu_harvest
            .iter()
            .filter_map(|cpu| match cpu.data_type {
                crate::collection::cpu::CpuDataType::Cpu(core_num) => Some(CpuMetric {
                    core_index: core_num as usize,
                    usage_percent: cpu.usage,
                    temperature: None,
                }),
                _ => None,
            })
            .collect();

        // Memory
        let memory_data = stored_data.ram_harvest.as_ref().map(|ram| MemoryMetric {
            used_bytes: ram.used_bytes,
            total_bytes: ram.total_bytes.get(),
            swap_used_bytes: stored_data.swap_harvest.as_ref().map(|s| s.used_bytes),
            swap_total_bytes: stored_data
                .swap_harvest
                .as_ref()
                .map(|s| s.total_bytes.get()),
        });

        // Network
        let network_data = vec![NetworkMetric {
            interface_name: "total".to_string(),
            rx_bytes: stored_data.network_harvest.rx,
            tx_bytes: stored_data.network_harvest.tx,
            rx_packets: None,
            tx_packets: None,
        }];

        // Disk
        let disk_data: Vec<DiskMetric> = stored_data
            .disk_harvest
            .iter()
            .map(|disk| DiskMetric {
                device_name: disk.name.clone(),
                mount_point: disk.mount_point.clone(),
                used_bytes: disk.used_bytes.unwrap_or(0),
                total_bytes: disk.total_bytes.unwrap_or(0),
                read_bytes: None,
                write_bytes: None,
            })
            .collect();

        // Temperature
        let temperature_data: Vec<TemperatureMetric> = stored_data
            .temp_data
            .iter()
            .filter_map(|temp| {
                temp.temperature.as_ref().map(|typed_temp| {
                    let celsius = match typed_temp {
                        crate::app::data::TypedTemperature::Celsius(c) => *c as f32,
                        crate::app::data::TypedTemperature::Kelvin(k) => *k as f32 - 273.15,
                        crate::app::data::TypedTemperature::Fahrenheit(f) => {
                            (*f as f32 - 32.0) * 5.0 / 9.0
                        }
                    };

                    TemperatureMetric {
                        sensor_name: temp.sensor.clone(),
                        sensor_type: "hardware".to_string(),
                        temperature_celsius: celsius,
                    }
                })
            })
            .collect();

        // GPU data - memoria per GPU
        #[cfg(feature = "gpu")]
        let gpu_data: Vec<GpuMetric> = stored_data
            .gpu_harvest
            .iter()
            .enumerate()
            .map(|(idx, (name, mem_data))| GpuMetric {
                gpu_id: idx as u32,
                name: name.clone(),
                usage_percent: None, // Non disponibile in gpu_harvest
                memory_used_bytes: Some(mem_data.used_bytes),
                temperature_celsius: None, // Già nelle temperature
            })
            .collect();

        #[cfg(not(feature = "gpu"))]
        let gpu_data: Vec<GpuMetric> = vec![];

        // Process data - limitato ai top N processi per evitare alta cardinalità
        let mut process_data: Vec<ProcessMetric> = stored_data
            .process_data
            .process_harvest
            .values()
            .filter(|proc| {
                // Apply process filter if provided
                if let Some(filter) = process_filter {
                    filter.should_include_process(&proc.name, proc.pid as u32)
                } else {
                    true // No filter, include all
                }
            })
            .map(|proc| ProcessMetric {
                pid: proc.pid as u32,
                name: proc.name.clone(),
                cpu_usage_percent: proc.cpu_usage_percent,
                memory_bytes: proc.mem_usage,
            })
            .collect();
        process_data.sort_by(|a, b| {
            b.cpu_usage_percent
                .partial_cmp(&a.cpu_usage_percent)
                .unwrap_or(std::cmp::Ordering::Equal)
        });
        process_data.truncate(10);

        Self {
            cpu_data,
            memory_data,
            network_data,
            disk_data,
            process_data,
            temperature_data,
            gpu_data,
        }
    }
}

impl SystemMetrics for BottomDataAdapter {
    fn get_cpu_data(&self) -> Option<&[CpuMetric]> {
        if self.cpu_data.is_empty() {
            None
        } else {
            Some(&self.cpu_data)
        }
    }

    fn get_memory_data(&self) -> Option<&MemoryMetric> {
        self.memory_data.as_ref()
    }

    fn get_network_data(&self) -> Option<&[NetworkMetric]> {
        if self.network_data.is_empty() {
            None
        } else {
            Some(&self.network_data)
        }
    }

    fn get_disk_data(&self) -> Option<&[DiskMetric]> {
        if self.disk_data.is_empty() {
            None
        } else {
            Some(&self.disk_data)
        }
    }

    fn get_process_data(&self) -> Option<&[ProcessMetric]> {
        if self.process_data.is_empty() {
            None
        } else {
            Some(&self.process_data)
        }
    }

    fn get_temperature_data(&self) -> Option<&[TemperatureMetric]> {
        if self.temperature_data.is_empty() {
            None
        } else {
            Some(&self.temperature_data)
        }
    }

    fn get_gpu_data(&self) -> Option<&[GpuMetric]> {
        if self.gpu_data.is_empty() {
            None
        } else {
            Some(&self.gpu_data)
        }
    }
}
