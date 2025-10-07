// src/opentelemetry/exporter.rs
#[cfg(feature = "opentelemetry")]
use opentelemetry::{
    global,
    metrics::{Counter, Gauge, Meter},
    KeyValue,
};
#[cfg(feature = "opentelemetry")]
use opentelemetry_otlp::WithExportConfig;
#[cfg(feature = "opentelemetry")]
use opentelemetry_sdk::{
    metrics::{
        PeriodicReader,
        SdkMeterProvider,
        reader::DefaultAggregationSelector,
        reader::DefaultTemporalitySelector,
    },
    runtime::Tokio,
    Resource,
};


use lazy_static::lazy_static;
//use std::collections::HashMap;
//use std::sync::Arc;
//use tokio::sync::RwLock;

lazy_static! {
    static ref TOKIO_RUNTIME: tokio::runtime::Runtime = {
        tokio::runtime::Runtime::new().expect("Failed to create Tokio runtime")
    };
}

use super::config::OpenTelemetryConfig;
use super::integration::OpenTelemetryError;

// Tipi di dati generici che si adattano a bottom
// Dovrai adattare questi ai tipi esistenti nel tuo progetto
pub trait SystemMetrics {
    fn get_cpu_data(&self) -> Option<&[CpuMetric]>;
    fn get_memory_data(&self) -> Option<&MemoryMetric>;
    fn get_network_data(&self) -> Option<&[NetworkMetric]>;
    fn get_disk_data(&self) -> Option<&[DiskMetric]>;
    fn get_process_data(&self) -> Option<&[ProcessMetric]>;
    fn get_temperature_data(&self) -> Option<&[TemperatureMetric]>;
    fn get_gpu_data(&self) -> Option<&[GpuMetric]>;
}

// Strutture generiche per le metriche - adatta questi ai tuoi tipi esistenti
#[derive(Debug, Clone)]
pub struct CpuMetric {
    pub core_index: usize,
    pub usage_percent: f32,
    pub temperature: Option<f32>,
}

#[derive(Debug, Clone)]
pub struct MemoryMetric {
    pub used_bytes: u64,
    pub total_bytes: u64,
    pub swap_used_bytes: Option<u64>,
    pub swap_total_bytes: Option<u64>,
}

#[derive(Debug, Clone)]
pub struct NetworkMetric {
    pub interface_name: String,
    pub rx_bytes: u64,
    pub tx_bytes: u64,
    pub rx_packets: Option<u64>,
    pub tx_packets: Option<u64>,
}

#[derive(Debug, Clone)]
pub struct DiskMetric {
    pub device_name: String,
    pub mount_point: String,
    pub used_bytes: u64,
    pub total_bytes: u64,
    pub read_bytes: Option<u64>,
    pub write_bytes: Option<u64>,
}

#[derive(Debug, Clone)]
pub struct ProcessMetric {
    pub pid: u32,
    pub name: String,
    pub cpu_usage_percent: f32,
    pub memory_bytes: u64,
}

#[derive(Debug, Clone)]
pub struct TemperatureMetric {
    pub sensor_name: String,
    pub sensor_type: String,
    pub temperature_celsius: f32,
}

#[derive(Debug, Clone)]
pub struct GpuMetric {
    pub gpu_id: u32,
    pub name: String,
    pub usage_percent: Option<f32>,
    pub memory_used_bytes: Option<u64>,
    pub temperature_celsius: Option<f32>,
}

pub struct OpenTelemetryExporter {
    #[allow(dead_code)]
    meter: Meter,
    metrics: OpenTelemetryMetrics,
    config: OpenTelemetryConfig,
}

pub struct OpenTelemetryMetrics {
    // CPU metrics
    cpu_usage: Gauge<f64>,
    cpu_temp: Gauge<f64>,
    
    // Memory metrics
    memory_usage: Gauge<u64>,
    memory_total: Gauge<u64>,
    swap_usage: Gauge<u64>,
    swap_total: Gauge<u64>,
    
    // Network metrics
    network_rx_bytes: Gauge<u64>,
    network_tx_bytes: Gauge<u64>,
    network_rx_packets: Gauge<u64>,
    network_tx_packets: Gauge<u64>,
    
    // Disk metrics
    disk_usage: Gauge<u64>,
    disk_total: Gauge<u64>,
    disk_read_bytes: Counter<u64>,
    disk_write_bytes: Counter<u64>,
    
    // Process metrics
    process_count: Gauge<u64>,
    process_cpu_usage: Gauge<f64>,
    process_memory_usage: Gauge<u64>,
    
    // Temperature metrics
    temperature: Gauge<f64>,
    
    // GPU metrics
    gpu_usage: Gauge<f64>,
    gpu_memory_usage: Gauge<u64>,
    gpu_temperature: Gauge<f64>,
}

impl OpenTelemetryExporter {
    pub async fn new(config: OpenTelemetryConfig) -> Result<Self, Box<dyn std::error::Error + Send + Sync>> {
        // Build resource with service info and custom attributes
        let mut resource_attributes = vec![
            KeyValue::new("service.name", config.service_name.clone()),
            KeyValue::new("service.version", config.service_version.clone()),
        ];
        
        for (key, value) in &config.resource_attributes {
            resource_attributes.push(KeyValue::new(key.clone(), value.clone()));
        }
        
        let resource = Resource::new(resource_attributes);
        
        // Configure OTLP exporter
        let exporter = opentelemetry_otlp::new_exporter()
            .tonic()
            .with_endpoint(&config.endpoint)
            .build_metrics_exporter(
                Box::new(DefaultAggregationSelector::new()),
                Box::new(DefaultTemporalitySelector::new())
            )?;

        // Create periodic reader with configured interval
        let reader = PeriodicReader::builder(exporter, Tokio)
            .with_interval(config.export_interval())
            .with_timeout(config.export_timeout())
            .build();
        
        // Create meter provider and set as global
        let provider = SdkMeterProvider::builder()
            .with_reader(reader)
            .with_resource(resource)
            .build();
        
        global::set_meter_provider(provider);
        
        // Get meter instance
        let meter = global::meter("bottom-system-monitor");
        
        // Create metrics
        let metrics = OpenTelemetryMetrics::new(&meter)?;
        
        Ok(Self {
            meter,
            metrics,
            config,
        })
    }
    
    pub async fn export_cpu_data(&self, cpu_data: &[CpuMetric]) -> Result<(), OpenTelemetryError> {
        if !self.config.metrics.cpu {
            return Ok(());
        }
        
        for (core_idx, cpu) in cpu_data.iter().enumerate() {
            let labels = [KeyValue::new("core", core_idx.to_string())];
            
            self.metrics.cpu_usage.record(
                cpu.usage_percent as f64,
                &labels,
            );
            
            if let Some(temp) = cpu.temperature {
                self.metrics.cpu_temp.record(
                    temp as f64,
                    &labels,
                );
            }
        }
        
        Ok(())
    }
    
    pub async fn export_memory_data(&self, mem_data: &MemoryMetric) -> Result<(), OpenTelemetryError> {
        if !self.config.metrics.memory {
            return Ok(());
        }
        
        let labels = &[];
        
        self.metrics.memory_usage.record(mem_data.used_bytes, labels);
        self.metrics.memory_total.record(mem_data.total_bytes, labels);
        
        if let (Some(swap_used), Some(swap_total)) = (mem_data.swap_used_bytes, mem_data.swap_total_bytes) {
            self.metrics.swap_usage.record(swap_used, labels);
            self.metrics.swap_total.record(swap_total, labels);
        }
        
        Ok(())
    }
    
    pub async fn export_network_data(&self, network_data: &[NetworkMetric]) -> Result<(), OpenTelemetryError> {
        if !self.config.metrics.network {
            return Ok(());
        }
        
        for net in network_data {
            let labels = [KeyValue::new("interface", net.interface_name.clone())];
            
            self.metrics.network_rx_bytes.record(net.rx_bytes, &labels);
            self.metrics.network_tx_bytes.record(net.tx_bytes, &labels);
            
            if let (Some(rx_packets), Some(tx_packets)) = (net.rx_packets, net.tx_packets) {
                self.metrics.network_rx_packets.record(rx_packets, &labels);
                self.metrics.network_tx_packets.record(tx_packets, &labels);
            }
        }
        
        Ok(())
    }
    
    pub async fn export_disk_data(&self, disk_data: &[DiskMetric]) -> Result<(), OpenTelemetryError> {
        if !self.config.metrics.disk {
            return Ok(());
        }
        
        for disk in disk_data {
            let labels = [
                KeyValue::new("device", disk.device_name.clone()),
                KeyValue::new("mount_point", disk.mount_point.clone()),
            ];
            
            self.metrics.disk_usage.record(disk.used_bytes, &labels);
            self.metrics.disk_total.record(disk.total_bytes, &labels);
            
            if let (Some(read_bytes), Some(write_bytes)) = (disk.read_bytes, disk.write_bytes) {
                self.metrics.disk_read_bytes.add(read_bytes, &labels);
                self.metrics.disk_write_bytes.add(write_bytes, &labels);
            }
        }
        
        Ok(())
    }
    
    pub async fn export_process_data(&self, process_data: &[ProcessMetric]) -> Result<(), OpenTelemetryError> {
        if !self.config.metrics.processes {
            return Ok(());
        }
        
        // Export process count
        self.metrics.process_count.record(process_data.len() as u64, &[]);
        
        // Export individual process metrics (be careful about cardinality)
        for process in process_data.iter().take(50) { // Limit to top 50 processes
            let labels = [
                KeyValue::new("pid", process.pid.to_string()),
                KeyValue::new("name", process.name.clone()),
            ];
            
            self.metrics.process_cpu_usage.record(process.cpu_usage_percent as f64, &labels);
            self.metrics.process_memory_usage.record(process.memory_bytes, &labels);
        }
        
        Ok(())
    }



    pub async fn export_temperature_data(&self, temp_data: &[TemperatureMetric]) -> Result<(), OpenTelemetryError> {
        if !self.config.metrics.temperature {
            return Ok(());
        }
        
        for temp in temp_data {
            let labels = [
                KeyValue::new("sensor", temp.sensor_name.clone()),
                KeyValue::new("type", temp.sensor_type.clone()),
            ];
            
            self.metrics.temperature.record(temp.temperature_celsius as f64, &labels);
        }
        
        Ok(())
    }
    
    pub async fn export_gpu_data(&self, gpu_data: &[GpuMetric]) -> Result<(), OpenTelemetryError> {
        if !self.config.metrics.gpu {
            return Ok(());
        }
        
        for gpu in gpu_data {
            let labels = [
                KeyValue::new("gpu_id", gpu.gpu_id.to_string()),  
                KeyValue::new("name", gpu.name.clone()),
            ];
            
            if let Some(usage) = gpu.usage_percent { 
                self.metrics.gpu_usage.record(usage as f64, &labels);
            }
            
            if let Some(memory_usage) = gpu.memory_used_bytes { 
                self.metrics.gpu_memory_usage.record(memory_usage, &labels);
            }
            
            if let Some(temperature) = gpu.temperature_celsius { 
                self.metrics.gpu_temperature.record(temperature as f64, &labels);
            }
        }
        
        Ok(())
    }

    // Metodo generico che usa il trait SystemMetrics
    pub async fn export_system_data<T: SystemMetrics>(&self, data: &T) -> Result<(), OpenTelemetryError> {
        // Export CPU data
        if let Some(cpu_data) = data.get_cpu_data() {
            self.export_cpu_data(cpu_data).await?;
        }
        
        // Export memory data
        if let Some(mem_data) = data.get_memory_data() {
            self.export_memory_data(mem_data).await?;
        }
        
        // Export network data
        if let Some(network_data) = data.get_network_data() {
            self.export_network_data(network_data).await?;
        }
        
        // Export disk data
        if let Some(disk_data) = data.get_disk_data() {
            self.export_disk_data(disk_data).await?;
        }
        
        // Export process data
        if let Some(process_data) = data.get_process_data() {
            self.export_process_data(process_data).await?;
        }
        
        // Export temperature data
        if let Some(temp_data) = data.get_temperature_data() {
            self.export_temperature_data(temp_data).await?;
        }
        
        // Export GPU data
        if let Some(gpu_data) = data.get_gpu_data() {
            self.export_gpu_data(gpu_data).await?;
        }
        Ok(())
    }
}

impl OpenTelemetryMetrics {
    fn new(meter: &Meter) -> Result<Self, Box<dyn std::error::Error + Send + Sync>> {
        Ok(Self {
            // CPU metrics
            cpu_usage: meter
                .f64_gauge("system_cpu_usage_percent")
                .with_description("CPU usage percentage per core")
                .init(),
            
            cpu_temp: meter
                .f64_gauge("system_cpu_temperature_celsius")
                .with_description("CPU temperature in Celsius")
                .init(),
            
            // Memory metrics
            memory_usage: meter
                .u64_gauge("system_memory_usage_bytes")
                .with_description("Memory usage in bytes")
                .init(),
                
            memory_total: meter
                .u64_gauge("system_memory_total_bytes")
                .with_description("Total memory in bytes")
                .init(),
                
            swap_usage: meter
                .u64_gauge("system_swap_usage_bytes")
                .with_description("Swap usage in bytes")
                .init(),
                
            swap_total: meter
                .u64_gauge("system_swap_total_bytes")
                .with_description("Total swap in bytes")
                .init(),
            
            // Network metrics
            network_rx_bytes: meter
                .u64_gauge("system_network_rx_bytes_rate")  // ← Cambia nome e tipo
                .with_description("Network receive rate in bytes per second")
                .init(),
                
            network_tx_bytes: meter
                .u64_gauge("system_network_tx_bytes_rate")  // ← Cambia nome e tipo
                .with_description("Network transmit rate in bytes per second")
                .init(),
                
            network_rx_packets: meter
                .u64_gauge("system_network_rx_packets_rate")  // ← Cambia nome e tipo
                .with_description("Network receive rate in packets per second")
                .init(),
                
            network_tx_packets: meter
                .u64_gauge("system_network_tx_packets_rate")  // ← Cambia nome e tipo
                .with_description("Network transmit rate in packets per second")
                .init(),
            
            // Disk metrics
            disk_usage: meter
                .u64_gauge("system_disk_usage_bytes")
                .with_description("Disk usage in bytes")
                .init(),
                
            disk_total: meter
                .u64_gauge("system_disk_total_bytes")
                .with_description("Total disk space in bytes")
                .init(),
                
            disk_read_bytes: meter
                .u64_counter("system_disk_read_bytes_total")
                .with_description("Total disk bytes read")
                .init(),
                
            disk_write_bytes: meter
                .u64_counter("system_disk_write_bytes_total")
                .with_description("Total disk bytes written")
                .init(),
            
            // Process metrics
            process_count: meter
                .u64_gauge("system_process_count")
                .with_description("Number of running processes")
                .init(),
                
            process_cpu_usage: meter
                .f64_gauge("system_process_cpu_usage_percent")
                .with_description("Process CPU usage percentage")
                .init(),
                
            process_memory_usage: meter
                .u64_gauge("system_process_memory_usage_bytes")
                .with_description("Process memory usage in bytes")
                .init(),
            
            // Temperature metrics
            temperature: meter
                .f64_gauge("system_temperature_celsius")
                .with_description("System temperature in Celsius")
                .init(),
            
            // GPU metrics
            gpu_usage: meter
                .f64_gauge("system_gpu_usage_percent")
                .with_description("GPU usage percentage")
                .init(),
                
            gpu_memory_usage: meter
                .u64_gauge("system_gpu_memory_usage_bytes")
                .with_description("GPU memory usage in bytes")
                .init(),
                
            gpu_temperature: meter
                .f64_gauge("system_gpu_temperature_celsius")
                .with_description("GPU temperature in Celsius")
                .init(),
        })
    }
}