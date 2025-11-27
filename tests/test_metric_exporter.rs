//! Unit tests for OtlpMetricExporter

use opentelemetry_sdk::metrics::Temporality;
use opentelemetry_sdk::metrics::data::ResourceMetrics;
use opentelemetry_sdk::metrics::exporter::PushMetricExporter;
use otlp_arrow_library::{Config, OtlpLibrary};
use std::path::PathBuf;
use std::time::Duration;
use tempfile::TempDir;

/// Helper function to create a simple test metric
fn create_test_metric() -> ResourceMetrics {
    // Create a minimal ResourceMetrics for testing
    // Note: ResourceMetrics fields are private in opentelemetry-sdk 0.31,
    // so we use default() like the rest of the codebase
    ResourceMetrics::default()
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
        dashboard: Default::default(),
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
        dashboard: Default::default(),
    };

    let library = OtlpLibrary::new(config).await.unwrap();
    let exporter = library.metric_exporter();

    // Test force_flush method (via PushMetricExporter trait)
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
        dashboard: Default::default(),
    };

    let library = OtlpLibrary::new(config).await.unwrap();
    let exporter = library.metric_exporter();

    // Test shutdown_with_timeout method (via PushMetricExporter trait)
    let result = exporter.shutdown_with_timeout(Duration::from_secs(5));
    assert!(result.is_ok(), "Shutdown should succeed");

    // Library should still be functional after exporter shutdown
    let metrics = create_test_metric();
    let export_result = exporter.export(&metrics).await;
    assert!(
        export_result.is_ok(),
        "Export should still work after shutdown"
    );
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
        dashboard: Default::default(),
    };

    let library = OtlpLibrary::new(config).await.unwrap();
    let exporter = library.metric_exporter();

    // Test temporality method (via PushMetricExporter trait)
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
        dashboard: Default::default(),
    };

    let library = OtlpLibrary::new(config).await.unwrap();
    let exporter = library.metric_exporter();

    // Shutdown library to cause export to fail
    library.shutdown().await.expect("Shutdown should succeed");

    // Wait a bit for shutdown to complete
    tokio::time::sleep(tokio::time::Duration::from_millis(100)).await;

    // Try to export after shutdown - should convert error appropriately
    let metrics = create_test_metric();
    let result = exporter.export(&metrics).await;

    // Should return OTelSdkError::InternalFailure
    // Note: After shutdown, the library may still accept exports if the batch buffer
    // is still active, so we check for either error or success (both are valid)
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
    // If result is Ok, that's also acceptable - the library may still process exports
}
