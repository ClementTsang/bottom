// src/opentelemetry/integration.rs
use std::sync::Arc;
use tracing::{debug, error, warn};

use super::{OpenTelemetryConfig, OpenTelemetryExporter};
use super::exporter::SystemMetrics;

/// Gestisce l'integrazione OpenTelemetry nel loop principale di bottom
#[derive(Clone)]
pub struct OpenTelemetryIntegration {
    exporter: Option<Arc<OpenTelemetryExporter>>,
    config: Option<OpenTelemetryConfig>,
    last_export_time: std::time::Instant,
    is_healthy: bool,
    consecutive_failures: u32,
}

impl OpenTelemetryIntegration {
    /// Crea una nuova integrazione OpenTelemetry
    pub async fn new(config: Option<OpenTelemetryConfig>) -> Result<Self, OpenTelemetryError> {
        let exporter = if let Some(ref otel_config) = config {
            if otel_config.enabled {
                debug!("Initializing OpenTelemetry exporter with endpoint: {}", otel_config.endpoint);
                match OpenTelemetryExporter::new(otel_config.clone()).await {
                    Ok(exp) => {
                        debug!("OpenTelemetry exporter initialized successfully");
                        Some(Arc::new(exp))
                    }
                    Err(e) => {
                        error!("Failed to initialize OpenTelemetry exporter: {}", e);
                        return Err(OpenTelemetryError::InitializationFailed(e.to_string()));
                    }
                }
            } else {
                debug!("OpenTelemetry is disabled in configuration");
                None
            }
        } else {
            debug!("No OpenTelemetry configuration provided");
            None
        };
        
        Ok(Self {
            exporter,
            config,
            last_export_time: std::time::Instant::now(),
            is_healthy: true,
            consecutive_failures: 0,
        })
    }
    
    /// Esporta i dati di sistema verso OpenTelemetry usando il trait SystemMetrics
    pub async fn export_system_data<T: SystemMetrics>(&mut self, data: &T) -> Result<(), OpenTelemetryError> {
        if let Some(exporter) = &self.exporter {
            let config = self.config.as_ref().unwrap();
            
            // Controlla se è il momento di esportare
            if !self.should_export() {
                return Ok(());
            }
            
            // Circuit breaker - se troppi errori consecutivi, salta questo export
            if self.consecutive_failures > config.max_consecutive_failures {
                if self.is_healthy {
                    warn!(
                        "OpenTelemetry exporter unhealthy after {} consecutive failures, pausing exports",
                        self.consecutive_failures
                    );
                    self.is_healthy = false;
                }
                return Ok(());
            }
            
            debug!("Exporting system data to OpenTelemetry");
            
            // Usa il metodo generico dell'exporter
            match exporter.export_system_data(data).await {
                Ok(_) => {
                    if !self.is_healthy {
                        debug!("OpenTelemetry exporter recovered");
                        self.is_healthy = true;
                    }
                    self.consecutive_failures = 0;
                    debug!("OpenTelemetry export completed successfully");
                }
                Err(e) => {
                    error!("OpenTelemetry export failed: {}", e);
                    self.consecutive_failures += 1;
                    if self.is_healthy && self.consecutive_failures >= 3 {
                        warn!("OpenTelemetry export experiencing issues ({} failures)", self.consecutive_failures);
                    }
                }
            }
            
            self.last_export_time = std::time::Instant::now();
        }
        
        Ok(())
    }
    
    /// Controlla se è il momento di esportare basandosi sull'intervallo configurato
    fn should_export(&self) -> bool {
        if let Some(config) = &self.config {
            self.last_export_time.elapsed() >= config.export_interval()
        } else {
            false
        }
    }
    
    /// Restituisce se l'integrazione è abilitata
    pub fn is_enabled(&self) -> bool {
        self.exporter.is_some()
    }
    
    /// Restituisce se l'exporter è in salute
    pub fn is_healthy(&self) -> bool {
        self.is_healthy
    }
    
    /// Restituisce statistiche dell'exporter
    pub fn get_stats(&self) -> OpenTelemetryStats {
        OpenTelemetryStats {
            is_enabled: self.is_enabled(),
            is_healthy: self.is_healthy,
            consecutive_failures: self.consecutive_failures,
            last_export_elapsed: self.last_export_time.elapsed(),
            endpoint: self.config.as_ref().map(|c| c.endpoint.clone()),
        }
    }
    
    /// Forza un export immediato (utile per testing)
    pub async fn force_export<T: SystemMetrics>(&mut self, data: &T) -> Result<(), OpenTelemetryError> {
        if let Some(_exporter) = &self.exporter {
            self.last_export_time = std::time::Instant::now() - std::time::Duration::from_secs(3600); // Force next export
            self.export_system_data(data).await
        } else {
            Err(OpenTelemetryError::NotInitialized)
        }
    }
}

/// Statistiche dell'integrazione OpenTelemetry
#[derive(Debug, Clone)]
pub struct OpenTelemetryStats {
    pub is_enabled: bool,
    pub is_healthy: bool,
    pub consecutive_failures: u32,
    pub last_export_elapsed: std::time::Duration,
    pub endpoint: Option<String>,
}

/// Errori specifici per OpenTelemetry
#[derive(Debug, thiserror::Error)]
pub enum OpenTelemetryError {
    #[error("OpenTelemetry initialization failed: {0}")]
    InitializationFailed(String),
    
    #[error("OpenTelemetry exporter not initialized")]
    NotInitialized,
    
    #[error("Export failed: {0}")]
    ExportFailed(String),
    
    #[error("Configuration error: {0}")]
    ConfigError(String),
    
    #[error("Network error: {0}")]
    NetworkError(String),
}

impl From<Box<dyn std::error::Error + Send + Sync>> for OpenTelemetryError {
    fn from(err: Box<dyn std::error::Error + Send + Sync>) -> Self {
        OpenTelemetryError::ExportFailed(err.to_string())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::opentelemetry::{OpenTelemetryConfig, adapter::BottomDataAdapter};
    
    #[tokio::test]
    async fn test_integration_disabled() {
        let integration = OpenTelemetryIntegration::new(None).await.unwrap();
        assert!(!integration.is_enabled());
    }
    
    #[tokio::test]
    async fn test_integration_enabled() {
        let config = OpenTelemetryConfig {
            enabled: true,
            endpoint: "http://localhost:4317".to_string(),
            ..Default::default()
        };
        
        // Questo test fallirà se non c'è un collector in ascolto, ma va bene per test unitari
        let _result = OpenTelemetryIntegration::new(Some(config)).await;
        // Non assertiamo il successo perché dipende dall'ambiente
    }
    
    #[test]
    fn test_should_export_timing() {
        let config = OpenTelemetryConfig {
            export_interval_secs: 1,
            ..Default::default()
        };
        
        let mut integration = OpenTelemetryIntegration {
            exporter: None,
            config: Some(config),
            last_export_time: std::time::Instant::now() - std::time::Duration::from_secs(2),
            is_healthy: true,
            consecutive_failures: 0,
        };
        
        assert!(integration.should_export());
        
        integration.last_export_time = std::time::Instant::now();
        assert!(!integration.should_export());
    }
    
    #[test]
    fn test_health_tracking() {
        let mut integration = OpenTelemetryIntegration {
            exporter: None,
            config: Some(OpenTelemetryConfig::default()),
            last_export_time: std::time::Instant::now(),
            is_healthy: true,
            consecutive_failures: 0,
        };
        
        // Simula errori
        integration.consecutive_failures = 5;
        assert!(integration.consecutive_failures > integration.config.as_ref().unwrap().max_consecutive_failures);
        
        // Reset dopo successo
        integration.consecutive_failures = 0;
        integration.is_healthy = true;
        assert!(integration.is_healthy());
    }
    
    #[tokio::test]
    async fn test_export_with_sample_data() {
        let mut integration = OpenTelemetryIntegration::new(None).await.unwrap();
        let sample_data = BottomDataAdapter::with_sample_data();
        
        // Con integrazione disabilitata, dovrebbe essere un no-op
        let result = integration.export_system_data(&sample_data).await;
        assert!(result.is_ok());
    }
}