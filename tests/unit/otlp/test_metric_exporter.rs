//! Unit tests for OtlpMetricExporter

use otlp_arrow_library::{Config, OtlpLibrary, OtlpMetricExporter};
use opentelemetry::KeyValue;
use opentelemetry_sdk::metrics::data::ResourceMetrics;
use opentelemetry_sdk::metrics::Temporality;
use opentelemetry_sdk::Resource;
use std::path::PathBuf;
use std::sync::Arc;
use std::time::Duration;
use tempfile::TempDir;

/// Helper function to create a simple test metric
fn create_test_metric() -> ResourceMetrics {
    ResourceMetrics {
        resource: Resource::new(vec![KeyValue::new("service.name", "test-service")]),
        scope_metrics: vec![],
    }
}

#[tokio::test]
async fn test_otlp_metric_exporter_export() {
    let temp_dir = TempDir::new().unwrap();
    
    let config = Config {
        output_dir: PathBuf::from(temp_dir.path()),
        write_interval_secs: 1,
        trace_cleanup_interval_secs: 600,
        metric_cleanup_interval_secs: 3600,
        protocols: Default::default(),
        forwarding: None,
    };

    let library = OtlpLibrary::new(config).await.unwrap();
    let exporter = library.metric_exporter();
    
    let metrics = create_test_metric();
    
    // Test export method
    let result = exporter.export(&metrics).await;
    assert!(result.is_ok(), "Export should succeed");
    
    // Flush to ensure write completes
    library.flush().await.expect("Failed to flush");
}

#[tokio::test]
async fn test_otlp_metric_exporter_force_flush() {
    let temp_dir = TempDir::new().unwrap();
    
    let config = Config {
        output_dir: PathBuf::from(temp_dir.path()),
        write_interval_secs: 1,
        trace_cleanup_interval_secs: 600,
        metric_cleanup_interval_secs: 3600,
        protocols: Default::default(),
        forwarding: None,
    };

    let library = OtlpLibrary::new(config).await.unwrap();
    let exporter = library.metric_exporter();
    
    // Test force_flush method
    let result = exporter.force_flush();
    assert!(result.is_ok(), "Force flush should succeed");
}

#[tokio::test]
async fn test_otlp_metric_exporter_shutdown_with_timeout() {
    let temp_dir = TempDir::new().unwrap();
    
    let config = Config {
        output_dir: PathBuf::from(temp_dir.path()),
        write_interval_secs: 1,
        trace_cleanup_interval_secs: 600,
        metric_cleanup_interval_secs: 3600,
        protocols: Default::default(),
        forwarding: None,
    };

    let library = OtlpLibrary::new(config).await.unwrap();
    let exporter = library.metric_exporter();
    
    // Test shutdown_with_timeout method (should return Ok immediately)
    let result = exporter.shutdown_with_timeout(Duration::from_secs(5));
    assert!(result.is_ok(), "Shutdown should succeed");
    
    // Library should still be functional after exporter shutdown
    let metrics = create_test_metric();
    let export_result = exporter.export(&metrics).await;
    assert!(export_result.is_ok(), "Export should still work after shutdown");
}

#[tokio::test]
async fn test_otlp_metric_exporter_temporality_returns_cumulative() {
    let temp_dir = TempDir::new().unwrap();
    
    let config = Config {
        output_dir: PathBuf::from(temp_dir.path()),
        write_interval_secs: 1,
        trace_cleanup_interval_secs: 600,
        metric_cleanup_interval_secs: 3600,
        protocols: Default::default(),
        forwarding: None,
    };

    let library = OtlpLibrary::new(config).await.unwrap();
    let exporter = library.metric_exporter();
    
    // Test temporality method
    let temporality = exporter.temporality();
    assert_eq!(
        temporality,
        Temporality::Cumulative,
        "Temporality should default to Cumulative"
    );
}

#[tokio::test]
async fn test_otlp_metric_exporter_error_conversion() {
    let temp_dir = TempDir::new().unwrap();
    
    let config = Config {
        output_dir: PathBuf::from(temp_dir.path()),
        write_interval_secs: 1,
        trace_cleanup_interval_secs: 600,
        metric_cleanup_interval_secs: 3600,
        protocols: Default::default(),
        forwarding: None,
    };

    let library = OtlpLibrary::new(config).await.unwrap();
    let exporter = library.metric_exporter();
    
    // Shutdown library to cause export to fail
    library.shutdown().await.expect("Shutdown should succeed");
    
    // Try to export after shutdown - should convert error appropriately
    let metrics = create_test_metric();
    let result = exporter.export(&metrics).await;
    
    // Should return OTelSdkError::InternalFailure
    assert!(result.is_err(), "Export should fail after library shutdown");
    if let Err(e) = result {
        // Verify it's an InternalFailure error
        match e {
            opentelemetry_sdk::error::OTelSdkError::InternalFailure(msg) => {
                assert!(
                    msg.contains("OtlpLibrary"),
                    "Error message should contain context about OtlpLibrary"
                );
            }
            _ => panic!("Expected InternalFailure error, got: {:?}", e),
        }
    }
}

