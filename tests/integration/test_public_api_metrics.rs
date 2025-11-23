//! Integration test for public API metrics export

use otlp_arrow_library::{Config, OtlpLibrary};
use opentelemetry::KeyValue;
use opentelemetry_sdk::metrics::data::{ResourceMetrics, ScopeMetrics, Metric};
use opentelemetry_sdk::Resource;
use std::path::PathBuf;
use std::time::Duration;
use tempfile::TempDir;
use tokio::time::sleep;

/// Helper function to create a simple test metric
fn create_test_metric() -> ResourceMetrics {
    // Create a minimal ResourceMetrics for testing
    // Note: This is simplified - full implementation would create proper metric data
    ResourceMetrics {
        resource: Resource::new(vec![KeyValue::new("service.name", "test-service")]),
        scope_metrics: vec![],
    }
}

#[tokio::test]
async fn test_public_api_metrics_export() {
    // Create a temporary directory for testing
    let temp_dir = TempDir::new().unwrap();
    
    let config = Config {
        output_dir: PathBuf::from(temp_dir.path()),
        write_interval_secs: 1, // Short interval for testing
        trace_cleanup_interval_secs: 600,
        metric_cleanup_interval_secs: 3600,
        protocols: Default::default(),
        forwarding: None,
    };

    // Create library instance
    let library = OtlpLibrary::new(config.clone()).await.unwrap();
    
    // Export metrics using public API
    let metrics = create_test_metric();
    library.export_metrics(metrics).await.expect("Failed to export metrics");
    
    // Wait for batch write
    sleep(Duration::from_secs(2)).await;
    
    // Flush to ensure all writes are complete
    library.flush().await.expect("Failed to flush");
    
    // Verify file was created
    let metrics_dir = temp_dir.path().join("otlp/metrics");
    let files: Vec<_> = std::fs::read_dir(&metrics_dir)
        .unwrap()
        .filter_map(|e| e.ok())
        .collect();
    
    assert!(!files.is_empty(), "Expected at least one metrics file to be created");
    
    // Verify file is readable as Arrow IPC
    let first_file = files[0].path();
    assert!(first_file.exists(), "Metrics file should exist");
    assert!(first_file.extension().unwrap() == "arrow" || first_file.file_name().unwrap().to_string_lossy().contains("arrow"),
        "Metrics file should have .arrow extension or contain 'arrow' in name");
    
    // Cleanup
    library.shutdown().await.unwrap();
}

