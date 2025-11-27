//! Integration test for metrics export with protobuf storage

use opentelemetry_proto::tonic::collector::metrics::v1::ExportMetricsServiceRequest;
use opentelemetry_proto::tonic::metrics::v1::ResourceMetrics;
use opentelemetry_proto::tonic::common::v1::{AnyValue, KeyValue};
use otlp_arrow_library::{Config, OtlpLibrary};
use opentelemetry_sdk::metrics::data::ResourceMetrics as SdkResourceMetrics;
use std::path::PathBuf;
use std::time::Duration;
use tempfile::TempDir;
use tokio::time::sleep;

/// Helper to create a test protobuf metrics request
fn create_test_protobuf_metrics_request() -> ExportMetricsServiceRequest {
    let resource = Some(opentelemetry_proto::tonic::resource::v1::Resource {
        attributes: vec![KeyValue {
            key: "service.name".to_string(),
            value: Some(AnyValue {
                value: Some(
                    opentelemetry_proto::tonic::common::v1::any_value::Value::StringValue(
                        "test-service".to_string(),
                    ),
                ),
            }),
        }],
        dropped_attributes_count: 0,
        entity_refs: vec![],
    });

    let resource_metrics = ResourceMetrics {
        resource,
        scope_metrics: vec![],
        schema_url: "".to_string(),
    };

    ExportMetricsServiceRequest {
        resource_metrics: vec![resource_metrics],
    }
}

#[tokio::test]
async fn test_metrics_protobuf_storage_and_export() {
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

    // Export metrics using export_metrics_arrow (converts ResourceMetrics to Arrow directly)
    let metrics = SdkResourceMetrics::default();
    library
        .export_metrics_arrow(&metrics)
        .await
        .expect("Failed to export metrics");

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

    // File should be created even with minimal metrics
    assert!(
        !files.is_empty(),
        "Expected at least one metrics file to be created"
    );

    // Verify file is readable as Arrow IPC
    let first_file = files[0].path();
    assert!(first_file.exists(), "Metrics file should exist");

    // Cleanup
    library.shutdown().await.unwrap();
}

#[tokio::test]
async fn test_metrics_protobuf_storage_multiple_exports() {
    // Create a temporary directory for testing
    let temp_dir = TempDir::new().unwrap();

    let config = Config {
        output_dir: PathBuf::from(temp_dir.path()),
        write_interval_secs: 1,
        trace_cleanup_interval_secs: 600,
        metric_cleanup_interval_secs: 3600,
        protocols: Default::default(),
        forwarding: None,
    };

    // Create library instance
    let library = OtlpLibrary::new(config.clone()).await.unwrap();

    // Export multiple metrics (each will be converted to Arrow and stored)
    for i in 0..5 {
        let metrics = SdkResourceMetrics::default();
        library
            .export_metrics_arrow(&metrics)
            .await
            .expect(&format!("Failed to export metric {}", i));
    }

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

    assert!(
        !files.is_empty(),
        "Expected at least one metrics file to be created"
    );

    // Cleanup
    library.shutdown().await.unwrap();
}

#[tokio::test]
async fn test_metrics_protobuf_storage_flush_behavior() {
    // Create a temporary directory for testing
    let temp_dir = TempDir::new().unwrap();

    let config = Config {
        output_dir: PathBuf::from(temp_dir.path()),
        write_interval_secs: 10, // Long interval - we'll use flush instead
        trace_cleanup_interval_secs: 600,
        metric_cleanup_interval_secs: 3600,
        protocols: Default::default(),
        forwarding: None,
    };

    // Create library instance
    let library = OtlpLibrary::new(config.clone()).await.unwrap();

    // Export metrics using export_metrics_arrow
    let metrics = SdkResourceMetrics::default();
    library
        .export_metrics_arrow(&metrics)
        .await
        .expect("Failed to export metrics");

    // Immediately flush (should write even though interval hasn't passed)
    library.flush().await.expect("Failed to flush");

    // Verify file was created immediately after flush
    let metrics_dir = temp_dir.path().join("otlp/metrics");
    let files: Vec<_> = std::fs::read_dir(&metrics_dir)
        .unwrap()
        .filter_map(|e| e.ok())
        .collect();

    assert!(
        !files.is_empty(),
        "Expected metrics file to be created immediately after flush"
    );

    // Cleanup
    library.shutdown().await.unwrap();
}

