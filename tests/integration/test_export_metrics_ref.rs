//! Integration test for export_metrics_arrow end-to-end flow

use otlp_arrow_library::{Config, OtlpLibrary};
use opentelemetry::KeyValue;
use opentelemetry_sdk::metrics::data::ResourceMetrics;
use opentelemetry_sdk::Resource;
use std::path::PathBuf;
use std::time::Duration;
use tempfile::TempDir;
use tokio::time::sleep;

/// Helper function to create a simple test metric
fn create_test_metric() -> ResourceMetrics {
    // Create a minimal ResourceMetrics for testing
    // Note: ResourceMetrics fields are private in opentelemetry-sdk 0.31,
    // so we use default() like the rest of the codebase
    ResourceMetrics::default()
}

#[tokio::test]
async fn test_export_metrics_arrow_end_to_end_flow() {
    // Create a temporary directory for testing
    let temp_dir = TempDir::new().unwrap();
    
    let config = Config {
        output_dir: PathBuf::from(temp_dir.path()),
        write_interval_secs: 1, // Short interval for testing
        trace_cleanup_interval_secs: 600,
        metric_cleanup_interval_secs: 3600,
        protocols: Default::default(),
        forwarding: None,
        dashboard: Default::default(),
    };

    // Create library instance
    let library = OtlpLibrary::new(config.clone()).await.unwrap();
    
    // Export metrics using reference method
    let metrics = create_test_metric();
    library.export_metrics_arrow(&metrics).await.expect("Failed to export metrics to Arrow");
    
    // Wait for batch write
    sleep(Duration::from_secs(2)).await;
    
    // Flush to ensure all writes are complete
    library.flush().await.expect("Failed to flush");
    
    // Verify file was created
    let metrics_dir = temp_dir.path().join("otlp/metrics");
    assert!(metrics_dir.exists(), "Metrics directory should exist");
    
    // Verify metrics were written (check for files in metrics directory)
    let entries: Vec<_> = std::fs::read_dir(&metrics_dir)
        .expect("Should be able to read metrics directory")
        .collect();
    assert!(!entries.is_empty(), "Metrics files should be created");
    
    // Shutdown gracefully
    library.shutdown().await.expect("Failed to shutdown");
}

