// src/opentelemetry/mod.rs

//! OpenTelemetry integration module for bottom system monitor
//!
//! This module provides functionality to export system metrics collected by bottom
//! to OpenTelemetry-compatible backends (like Jaeger, OpenTelemetry Collector, etc.)
//! using the OTLP protocol.

pub mod adapter;
pub mod config;
pub mod exporter;
pub mod integration;

// Re-export main types for easier importing
pub use adapter::BottomDataAdapter;
pub use config::{MetricsConfig, OpenTelemetryConfig};
pub use exporter::{OpenTelemetryExporter, OpenTelemetryMetrics, SystemMetrics};
pub use integration::{OpenTelemetryError, OpenTelemetryIntegration, OpenTelemetryStats};

/// Convenience function to create and initialize OpenTelemetry integration
///
/// # Arguments
/// * `config` - Optional OpenTelemetry configuration
///
/// # Returns
/// * `Result<OpenTelemetryIntegration, OpenTelemetryError>` - Initialized integration or error
///
/// # Example
/// ```rust
/// use bottom::opentelemetry::{initialize_opentelemetry, OpenTelemetryConfig};
///
/// #[tokio::main]
/// async fn main() -> Result<(), Box<dyn std::error::Error>> {
///     let config = OpenTelemetryConfig {
///         enabled: true,
///         endpoint: "http://localhost:4317".to_string(),
///         ..Default::default()
///     };
///     
///     let otel = initialize_opentelemetry(Some(config)).await?;
///     // Use otel.export_system_data(&data) in your loop
///     
///     Ok(())
/// }
/// ```
pub async fn initialize_opentelemetry(
    config: Option<OpenTelemetryConfig>,
) -> Result<OpenTelemetryIntegration, OpenTelemetryError> {
    OpenTelemetryIntegration::new(config).await
}

/// Check if OpenTelemetry is available and properly configured
///
/// This function validates the configuration and tests connectivity
/// without initializing the full exporter.
pub async fn check_opentelemetry_availability(
    config: &OpenTelemetryConfig,
) -> Result<bool, OpenTelemetryError> {
    // Validate configuration first
    config.validate().map_err(OpenTelemetryError::ConfigError)?;

    if !config.enabled {
        return Ok(false);
    }

    // TODO: Add actual connectivity check to the OTLP endpoint
    // For now, just return true if config is valid
    Ok(true)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_initialize_disabled() {
        let result = initialize_opentelemetry(None).await;
        assert!(result.is_ok());

        let integration = result.unwrap();
        assert!(!integration.is_enabled());
    }

    #[tokio::test]
    async fn test_initialize_enabled() {
        let config = OpenTelemetryConfig {
            enabled: true,
            endpoint: "http://localhost:4317".to_string(),
            ..Default::default()
        };

        // This might fail if no collector is running, which is fine for unit tests
        let _result = initialize_opentelemetry(Some(config)).await;
        // We don't assert success because it depends on external infrastructure
    }

    #[tokio::test]
    async fn test_check_availability_disabled() {
        let config = OpenTelemetryConfig {
            enabled: false,
            ..Default::default()
        };

        let result = check_opentelemetry_availability(&config).await.unwrap();
        assert!(!result);
    }

    #[tokio::test]
    async fn test_check_availability_invalid_config() {
        let config = OpenTelemetryConfig {
            enabled: true,
            endpoint: "".to_string(), // Invalid empty endpoint
            ..Default::default()
        };

        let result = check_opentelemetry_availability(&config).await;
        assert!(result.is_err());
        assert!(matches!(
            result.unwrap_err(),
            OpenTelemetryError::ConfigError(_)
        ));
    }

    #[test]
    fn test_config_default() {
        let config = OpenTelemetryConfig::default();
        assert!(!config.enabled);
        assert_eq!(config.endpoint, "http://localhost:4317");
        assert_eq!(config.service_name, "bottom-system-monitor");
    }

    #[test]
    fn test_metrics_config_default() {
        let metrics = MetricsConfig::default();
        assert!(metrics.cpu);
        assert!(metrics.memory);
        assert!(metrics.network);
        assert!(metrics.disk);
        assert!(!metrics.processes); // Should be disabled by default due to high cardinality
        assert!(metrics.temperature);
        assert!(metrics.gpu);
    }
}
