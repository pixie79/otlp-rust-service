//! Integration test for custom write interval configuration

use otlp_arrow_library::api::OtlpLibrary;
use otlp_arrow_library::config::ConfigBuilder;
use opentelemetry::trace::{SpanContext, SpanId, SpanKind, Status, TraceId};
use opentelemetry::KeyValue;
use opentelemetry_sdk::trace::SpanData;
use std::time::{Duration, SystemTime};
use tempfile::TempDir;
use tokio::time::{sleep, Instant};

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
async fn test_custom_write_interval() {
    let temp_dir = TempDir::new().unwrap();
    
    // Create config with custom write interval (2 seconds)
    let config = ConfigBuilder::new()
        .output_dir(temp_dir.path())
        .write_interval_secs(2)
        .build()
        .unwrap();
    
    // Create library instance
    let library = OtlpLibrary::new(config).await.unwrap();
    
    // Export a trace
    let span = create_test_span("test-span-interval");
    library.export_trace(span).await.unwrap();
    
    // Check that file doesn't exist immediately
    let traces_dir = temp_dir.path().join("otlp/traces");
    let initial_count = if traces_dir.exists() {
        std::fs::read_dir(&traces_dir).unwrap().count()
    } else {
        0
    };
    
    // Wait for write interval (2 seconds + buffer)
    let start = Instant::now();
    sleep(Duration::from_millis(2200)).await;
    let elapsed = start.elapsed();
    
    // Verify that at least 2 seconds passed
    assert!(elapsed >= Duration::from_secs(2), "Should wait at least 2 seconds");
    
    // Flush to ensure write
    library.flush().await.unwrap();
    
    // Verify files were created after the interval
    assert!(traces_dir.exists(), "Traces directory should exist");
    let final_count = std::fs::read_dir(&traces_dir).unwrap().count();
    assert!(final_count > initial_count, "Should have new files after write interval");
    
    // Cleanup
    library.shutdown().await.unwrap();
}

#[tokio::test]
async fn test_default_write_interval() {
    let temp_dir = TempDir::new().unwrap();
    
    // Create config with default write interval (5 seconds)
    let config = ConfigBuilder::new()
        .output_dir(temp_dir.path())
        // Don't set write_interval_secs - should use default of 5
        .build()
        .unwrap();
    
    assert_eq!(config.write_interval_secs, 5, "Should use default write interval of 5 seconds");
    
    // Create library instance
    let library = OtlpLibrary::new(config).await.unwrap();
    
    // Export a trace
    let span = create_test_span("test-span-default-interval");
    library.export_trace(span).await.unwrap();
    
    // Wait less than default interval (3 seconds < 5 seconds)
    sleep(Duration::from_millis(3100)).await;
    
    // Flush to ensure write
    library.flush().await.unwrap();
    
    // Verify files were created
    let traces_dir = temp_dir.path().join("otlp/traces");
    assert!(traces_dir.exists(), "Traces directory should exist");
    
    // Cleanup
    library.shutdown().await.unwrap();
}

#[tokio::test]
async fn test_very_short_write_interval() {
    let temp_dir = TempDir::new().unwrap();
    
    // Create config with very short write interval (100ms)
    let config = ConfigBuilder::new()
        .output_dir(temp_dir.path())
        .write_interval_secs(0) // This should fail validation, but let's test with 1 second minimum
        .build();
    
    // Should fail validation for zero interval
    assert!(config.is_err());
    
    // Test with minimum valid interval (1 second)
    let config = ConfigBuilder::new()
        .output_dir(temp_dir.path())
        .write_interval_secs(1)
        .build()
        .unwrap();
    
    let library = OtlpLibrary::new(config).await.unwrap();
    
    // Export multiple traces
    for i in 0..3 {
        let span = create_test_span(&format!("test-span-{}", i));
        library.export_trace(span).await.unwrap();
    }
    
    // Wait for write interval
    sleep(Duration::from_millis(1200)).await;
    
    // Flush to ensure write
    library.flush().await.unwrap();
    
    // Verify files were created
    let traces_dir = temp_dir.path().join("otlp/traces");
    assert!(traces_dir.exists(), "Traces directory should exist");
    
    // Cleanup
    library.shutdown().await.unwrap();
}

