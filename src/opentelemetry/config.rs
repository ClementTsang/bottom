// src/opentelemetry/config.rs
use regex::Regex;
use serde::{Deserialize, Serialize};
use std::path::PathBuf;
use std::time::Duration;

#[derive(Debug, Clone, Serialize, Deserialize)]
#[cfg_attr(feature = "generate_schema", derive(schemars::JsonSchema))]
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
#[cfg_attr(feature = "generate_schema", derive(schemars::JsonSchema))]
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

    /// Process filter configuration (inline or via include)
    #[serde(default)]
    pub process_filter: ProcessFilterConfig,
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
#[cfg_attr(feature = "generate_schema", derive(schemars::JsonSchema))]
pub struct ProcessFilterConfig {
    /// Path to external file containing process filter (optional)
    /// If specified, will load filter configuration from this file
    #[serde(skip_serializing_if = "Option::is_none")]
    pub include: Option<PathBuf>,

    /// Filter mode: "whitelist" (only listed processes) or "blacklist" (exclude listed)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub filter_mode: Option<ProcessFilterMode>,

    /// List of process names to filter (case-insensitive substring match)
    #[serde(default)]
    pub names: Vec<String>,

    /// List of regex patterns to match process names
    #[serde(default)]
    pub patterns: Vec<String>,

    /// List of process PIDs to filter
    #[serde(default)]
    pub pids: Vec<u32>,

    /// Compiled regex patterns (not serialized, built at runtime)
    #[serde(skip)]
    #[cfg_attr(feature = "generate_schema", schemars(skip))]
    compiled_patterns: Option<Vec<Regex>>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[cfg_attr(feature = "generate_schema", derive(schemars::JsonSchema))]
#[serde(rename_all = "lowercase")]
pub enum ProcessFilterMode {
    Whitelist,
    Blacklist,
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
            process_filter: ProcessFilterConfig::default(),
        }
    }
}

impl ProcessFilterConfig {
    /// Load and merge process filter from include file if specified
    pub fn load_with_includes(&self, config_dir: Option<&std::path::Path>) -> Result<Self, String> {
        if let Some(include_path) = &self.include {
            // Resolve path relative to config directory if provided
            let full_path = if include_path.is_absolute() {
                include_path.clone()
            } else if let Some(dir) = config_dir {
                dir.join(include_path)
            } else {
                include_path.clone()
            };

            // Read and parse the included file
            let content = std::fs::read_to_string(&full_path).map_err(|e| {
                format!("Failed to read process filter file {:?}: {}", full_path, e)
            })?;

            let included: ProcessFilterConfig = toml_edit::de::from_str(&content).map_err(|e| {
                format!("Failed to parse process filter file {:?}: {}", full_path, e)
            })?;

            // Merge: included file takes precedence, but combine lists
            let mut merged = Self {
                include: None, // Don't recurse
                filter_mode: included.filter_mode.or(self.filter_mode.clone()),
                names: if included.names.is_empty() {
                    self.names.clone()
                } else {
                    included.names
                },
                patterns: if included.patterns.is_empty() {
                    self.patterns.clone()
                } else {
                    included.patterns
                },
                pids: if included.pids.is_empty() {
                    self.pids.clone()
                } else {
                    included.pids
                },
                compiled_patterns: None,
            };

            // Compile regex patterns
            merged.compile_patterns()?;
            Ok(merged)
        } else {
            // No include, compile patterns for self
            let mut result = self.clone();
            result.compile_patterns()?;
            Ok(result)
        }
    }

    /// Compile regex patterns from strings
    fn compile_patterns(&mut self) -> Result<(), String> {
        if self.patterns.is_empty() {
            self.compiled_patterns = None;
            return Ok(());
        }

        let mut compiled = Vec::new();
        for pattern in &self.patterns {
            match Regex::new(pattern) {
                Ok(regex) => compiled.push(regex),
                Err(e) => {
                    return Err(format!("Invalid regex pattern '{}': {}", pattern, e));
                }
            }
        }

        self.compiled_patterns = Some(compiled);
        Ok(())
    }

    /// Check if a process should be included based on filter configuration
    pub fn should_include_process(&self, process_name: &str, process_pid: u32) -> bool {
        // If no filter mode is set, include all processes
        let filter_mode = match &self.filter_mode {
            Some(mode) => mode,
            None => return true,
        };

        // Check if process matches the filter lists
        let matches_name = self
            .names
            .iter()
            .any(|name| process_name.to_lowercase().contains(&name.to_lowercase()));

        // Check if process matches regex patterns
        let matches_pattern = if let Some(patterns) = &self.compiled_patterns {
            patterns.iter().any(|regex| regex.is_match(process_name))
        } else {
            false
        };

        let matches_pid = self.pids.contains(&process_pid);
        let matches = matches_name || matches_pattern || matches_pid;

        // Apply filter mode logic
        match filter_mode {
            ProcessFilterMode::Whitelist => matches, // Include only if matches
            ProcessFilterMode::Blacklist => !matches, // Include only if NOT matches
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
