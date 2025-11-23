//! Integration test for configuration with defaults

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
async fn test_config_with_all_defaults() {
    let temp_dir = TempDir::new().unwrap();
    
    // Create config with only output_dir specified, all others should use defaults
    let config = ConfigBuilder::new()
        .output_dir(temp_dir.path())
        .build()
        .unwrap();
    
    // Verify all default values
    assert_eq!(config.output_dir, temp_dir.path());
    assert_eq!(config.write_interval_secs, 5, "Should use default write interval");
    assert_eq!(config.trace_cleanup_interval_secs, 600, "Should use default trace cleanup interval");
    assert_eq!(config.metric_cleanup_interval_secs, 3600, "Should use default metric cleanup interval");
    
    // Verify default protocol settings
    assert!(config.protocols.protobuf_enabled, "Protobuf should be enabled by default");
    assert!(config.protocols.arrow_flight_enabled, "Arrow Flight should be enabled by default");
    assert_eq!(config.protocols.protobuf_port, 4317, "Should use default Protobuf port");
    assert_eq!(config.protocols.arrow_flight_port, 4318, "Should use default Arrow Flight port");
    
    // Verify forwarding is disabled by default
    assert!(config.forwarding.is_none() || !config.forwarding.as_ref().unwrap().enabled, 
            "Forwarding should be disabled by default");
    
    // Create library instance and verify it works with defaults
    let library = OtlpLibrary::new(config).await.unwrap();
    
    // Export a trace
    let span = create_test_span("test-span-defaults");
    library.export_trace(span).await.unwrap();
    
    // Wait for default write interval (5 seconds)
    sleep(Duration::from_millis(5200)).await;
    
    // Flush to ensure write
    library.flush().await.unwrap();
    
    // Verify files were created
    let traces_dir = temp_dir.path().join("otlp/traces");
    assert!(traces_dir.exists(), "Traces directory should exist");
    
    // Cleanup
    library.shutdown().await.unwrap();
}

#[tokio::test]
async fn test_config_default_output_dir() {
    // Test that default output directory is used when not specified
    let config = ConfigBuilder::new()
        .build()
        .unwrap();
    
    // Default output dir should be ./output_dir
    assert_eq!(config.output_dir, PathBuf::from("./output_dir"), 
               "Should use default output directory");
}

#[tokio::test]
async fn test_config_partial_customization() {
    let temp_dir = TempDir::new().unwrap();
    
    // Create config with some custom values and some defaults
    let config = ConfigBuilder::new()
        .output_dir(temp_dir.path())
        .write_interval_secs(10) // Custom
        // trace_cleanup_interval_secs - use default
        // metric_cleanup_interval_secs - use default
        .build()
        .unwrap();
    
    // Verify custom value
    assert_eq!(config.write_interval_secs, 10);
    
    // Verify defaults are still used
    assert_eq!(config.trace_cleanup_interval_secs, 600);
    assert_eq!(config.metric_cleanup_interval_secs, 3600);
    
    // Verify protocol defaults
    assert!(config.protocols.protobuf_enabled);
    assert!(config.protocols.arrow_flight_enabled);
}

