//! Integration test for custom output directory configuration

use otlp_arrow_library::api::OtlpLibrary;
use otlp_arrow_library::config::ConfigBuilder;
use opentelemetry::trace::{SpanContext, SpanId, SpanKind, Status, TraceId};
use opentelemetry::KeyValue;
use opentelemetry_sdk::trace::SpanData;
use std::path::PathBuf;
use std::time::{Duration, SystemTime};
use tempfile::TempDir;
use tokio::time::sleep;

/// Helper function to create a test span
fn create_test_span(name: &str) -> SpanData {
    let trace_id = TraceId::from_bytes([1, 2, 3, 4, 5, 6, 7, 8, 9, 10, 11, 12, 13, 14, 15, 16]);
    let span_id = SpanId::from_bytes([1, 2, 3, 4, 5, 6, 7, 8]);
    let parent_span_id = SpanId::from_bytes([9, 10, 11, 12, 13, 14, 15, 16]);
    
    let span_context = SpanContext::new(trace_id, span_id, 0, false);
    
    SpanData {
        span_context,
        parent_span_id,
        span_kind: SpanKind::Server,
        name: name.to_string().into(),
        start_time: SystemTime::now(),
        end_time: SystemTime::now() + Duration::from_secs(1),
        attributes: vec![
            KeyValue::new("service.name", "test-service"),
        ],
        events: vec![],
        links: vec![],
        status: Status::Ok,
        resource: opentelemetry_sdk::Resource::builder_empty().build(),
        dropped_attributes_count: 0,
        parent_span_is_remote: false,
        instrumentation_scope: opentelemetry::InstrumentationScope::builder("test")
            .with_version("1.0.0")
            .build(),
    }
}

#[tokio::test]
async fn test_custom_output_directory() {
    let temp_dir = TempDir::new().unwrap();
    let custom_output_dir = temp_dir.path().join("custom_output");
    
    // Create config with custom output directory
    let config = ConfigBuilder::new()
        .output_dir(&custom_output_dir)
        .write_interval_secs(1) // Short interval for faster test
        .build()
        .unwrap();
    
    // Create library instance
    let library = OtlpLibrary::new(config).await.unwrap();
    
    // Export a trace
    let span = create_test_span("test-span");
    library.export_trace(span).await.unwrap();
    
    // Wait for write interval
    sleep(Duration::from_millis(1200)).await;
    
    // Flush to ensure write
    library.flush().await.unwrap();
    
    // Verify files were created in custom directory
    let traces_dir = custom_output_dir.join("otlp/traces");
    assert!(traces_dir.exists(), "Traces directory should exist in custom output dir");
    
    // Check that trace files exist
    let entries: Vec<_> = std::fs::read_dir(&traces_dir)
        .unwrap()
        .collect();
    assert!(!entries.is_empty(), "Should have at least one trace file");
    
    // Verify the file is in the correct location
    for entry in entries {
        let entry = entry.unwrap();
        let path = entry.path();
        assert!(path.starts_with(&traces_dir), "File should be in custom output directory");
        assert!(path.to_string_lossy().contains("traces"), "File should be in traces subdirectory");
    }
}

#[tokio::test]
async fn test_default_output_directory() {
    let temp_dir = TempDir::new().unwrap();
    std::env::set_current_dir(temp_dir.path()).unwrap();
    
    // Create config with default output directory (relative path)
    let config = ConfigBuilder::new()
        .output_dir("./default_output")
        .write_interval_secs(1)
        .build()
        .unwrap();
    
    // Create library instance
    let library = OtlpLibrary::new(config.clone()).await.unwrap();
    
    // Export a trace
    let span = create_test_span("test-span-default");
    library.export_trace(span).await.unwrap();
    
    // Wait for write interval
    sleep(Duration::from_millis(1200)).await;
    
    // Flush to ensure write
    library.flush().await.unwrap();
    
    // Verify files were created in default directory (relative to current dir)
    let expected_dir = temp_dir.path().join("default_output/otlp/traces");
    assert!(expected_dir.exists(), "Traces directory should exist in default output dir");
    
    // Cleanup
    library.shutdown().await.unwrap();
}

