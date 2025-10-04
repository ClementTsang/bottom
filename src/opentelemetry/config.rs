// src/opentelemetry/config.rs
use serde::{Deserialize, Serialize};
use std::time::Duration;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct OpenTelemetryConfig {
    /// Enable OpenTelemetry metrics export
    #[serde(default)]
    pub enabled: bool,
    
    /// OTLP endpoint (e.g., "http://localhost:4317")
    #[serde(default = "default_endpoint")]
    pub endpoint: String,
    
    /// Export interval in seconds
    #[serde(default = "default_export_interval")]
    pub export_interval_secs: u64,
    
    /// Service name for the metrics
    #[serde(default = "default_service_name")]
    pub service_name: String,
    
    /// Service version
    #[serde(default = "default_service_version")]
    pub service_version: String,
    
    /// Additional resource attributes
    #[serde(default)]
    pub resource_attributes: std::collections::HashMap<String, String>,
    
    /// Metrics to export configuration
    #[serde(default)]
    pub metrics: MetricsConfig,
    
    /// Maximum consecutive failures before marking as unhealthy
    #[serde(default = "default_max_failures")]
    pub max_consecutive_failures: u32,
    
    /// Timeout for export operations in seconds
    #[serde(default = "default_timeout")]
    pub export_timeout_secs: u64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MetricsConfig {
    /// Export CPU metrics
    #[serde(default = "default_true")]
    pub cpu: bool,
    
    /// Export memory metrics
    #[serde(default = "default_true")]
    pub memory: bool,
    
    /// Export network metrics
    #[serde(default = "default_true")]
    pub network: bool,
    
    /// Export disk metrics
    #[serde(default = "default_true")]
    pub disk: bool,
    
    /// Export process metrics
    #[serde(default)]
    pub processes: bool,
    
    /// Export temperature metrics
    #[serde(default = "default_true")]
    pub temperature: bool,
    
    /// Export GPU metrics
    #[serde(default = "default_true")]
    pub gpu: bool,
}

// Default functions
fn default_endpoint() -> String {
    "http://localhost:4317".to_string()
}

fn default_export_interval() -> u64 {
    10
}

fn default_service_name() -> String {
    "bottom-system-monitor".to_string()
}

fn default_service_version() -> String {
    env!("CARGO_PKG_VERSION").to_string()
}

fn default_max_failures() -> u32 {
    5
}

fn default_timeout() -> u64 {
    30
}

fn default_true() -> bool {
    true
}

impl Default for OpenTelemetryConfig {
    fn default() -> Self {
        Self {
            enabled: false,
            endpoint: default_endpoint(),
            export_interval_secs: default_export_interval(),
            service_name: default_service_name(),
            service_version: default_service_version(),
            resource_attributes: std::collections::HashMap::new(),
            metrics: MetricsConfig::default(),
            max_consecutive_failures: default_max_failures(),
            export_timeout_secs: default_timeout(),
        }
    }
}

impl Default for MetricsConfig {
    fn default() -> Self {
        Self {
            cpu: true,
            memory: true,
            network: true,
            disk: true,
            processes: false,
            temperature: true,
            gpu: true,
        }
    }
}

impl OpenTelemetryConfig {
    pub fn export_interval(&self) -> Duration {
        Duration::from_secs(self.export_interval_secs)
    }
    
    pub fn export_timeout(&self) -> Duration {
        Duration::from_secs(self.export_timeout_secs)
    }
    
    /// Valida la configurazione
    pub fn validate(&self) -> Result<(), String> {
        if self.enabled {
            if self.endpoint.is_empty() {
                return Err("OpenTelemetry endpoint cannot be empty when enabled".to_string());
            }
            
            if self.export_interval_secs == 0 {
                return Err("Export interval must be greater than 0".to_string());
            }
            
            if self.service_name.is_empty() {
                return Err("Service name cannot be empty".to_string());
            }
            
            if !self.endpoint.starts_with("http://") && !self.endpoint.starts_with("https://") {
                return Err("OpenTelemetry endpoint must be a valid HTTP/HTTPS URL".to_string());
            }
        }
        
        Ok(())
    }
}