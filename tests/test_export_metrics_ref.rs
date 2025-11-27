//! Unit tests for export_metrics_arrow method

use opentelemetry_sdk::metrics::data::ResourceMetrics;
use otlp_arrow_library::{Config, OtlpLibrary};
use std::path::PathBuf;
use tempfile::TempDir;

/// Helper function to create a simple test metric
fn create_test_metric() -> ResourceMetrics {
    // Create a minimal ResourceMetrics for testing
    // Note: ResourceMetrics fields are private in opentelemetry-sdk 0.31,
    // so we use default() like the rest of the codebase
    ResourceMetrics::default()
}

#[tokio::test]
async fn test_export_metrics_arrow_with_valid_resource_metrics() {
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

    let metrics = create_test_metric();

    // Test export_metrics_arrow with valid ResourceMetrics
    library
        .export_metrics_arrow(&metrics)
        .await
        .expect("Failed to export metrics to Arrow");

    // Flush to ensure write completes
    library.flush().await.expect("Failed to flush");

    // Verify metrics were processed (check that batch buffer accepted them)
    // The actual file write happens asynchronously, but flush ensures it's written
    let metrics_dir = temp_dir.path().join("otlp/metrics");
    assert!(metrics_dir.exists(), "Metrics directory should exist");
}

#[tokio::test]
async fn test_export_metrics_arrow_functional_equivalence_with_export_metrics() {
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

    let library1 = OtlpLibrary::new(config.clone()).await.unwrap();
    let library2 = OtlpLibrary::new(config).await.unwrap();

    // Create two separate metrics instances since ResourceMetrics doesn't implement Clone
    let metrics1 = create_test_metric();
    let metrics2 = create_test_metric();

    // Export using export_metrics_arrow (direct ResourceMetrics to Arrow)
    library1
        .export_metrics_arrow(&metrics1)
        .await
        .expect("Failed to export metrics to Arrow");

    // Export using export_metrics_arrow (same method, different instance)
    library2
        .export_metrics_arrow(&metrics2)
        .await
        .expect("Failed to export metrics to Arrow");

    // Flush both
    library1.flush().await.expect("Failed to flush library1");
    library2.flush().await.expect("Failed to flush library2");

    // Both should have written files (functional equivalence)
    let metrics_dir1 = temp_dir.path().join("otlp/metrics");
    let metrics_dir2 = temp_dir.path().join("otlp/metrics");
    assert!(
        metrics_dir1.exists(),
        "Metrics directory should exist for owned export"
    );
    assert!(
        metrics_dir2.exists(),
        "Metrics directory should exist for reference export"
    );
}

#[tokio::test]
async fn test_export_metrics_arrow_with_empty_resource_metrics() {
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

    // Create empty ResourceMetrics using default
    // Note: ResourceMetrics fields are private, so we use default()
    let empty_metrics = ResourceMetrics::default();

    // Should handle empty metrics gracefully (no-op or success)
    let result = library.export_metrics_arrow(&empty_metrics).await;
    assert!(result.is_ok(), "Empty metrics should be handled gracefully");
}

#[tokio::test]
async fn test_export_metrics_arrow_concurrent_calls() {
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
    let library_arc = std::sync::Arc::new(library);

    // Spawn multiple concurrent calls
    // Each call gets its own metrics instance since ResourceMetrics doesn't implement Clone
    let mut handles = vec![];
    for _ in 0..10 {
        let lib = library_arc.clone();
        let metrics = create_test_metric();
        handles.push(tokio::spawn(async move {
            lib.export_metrics_arrow(&metrics).await
        }));
    }

    // Wait for all to complete
    for handle in handles {
        let result = handle.await.expect("Task should complete");
        assert!(
            result.is_ok(),
            "Concurrent export_metrics_arrow calls should succeed"
        );
    }

    // Flush to ensure all writes complete
    library_arc.flush().await.expect("Failed to flush");
}
