//! Integration test for public API trace export

use otlp_arrow_library::{Config, OtlpLibrary};
use opentelemetry::trace::{SpanContext, SpanId, SpanKind, Status, TraceId, TraceFlags, TraceState};
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
    
    let span_context = SpanContext::new(trace_id, span_id, TraceFlags::default(), false, TraceState::default());
    
    SpanData {
        span_context,
        parent_span_id,
        span_kind: SpanKind::Server,
        name: std::borrow::Cow::Owned(name.to_string()),
        start_time: SystemTime::now(),
        end_time: SystemTime::now() + Duration::from_secs(1),
        attributes: vec![
            KeyValue::new("service.name", "test-service"),
            KeyValue::new("http.method", "GET"),
        ],
        events: opentelemetry_sdk::trace::SpanEvents::default(),
        links: opentelemetry_sdk::trace::SpanLinks::default(),
        status: Status::Ok,
        dropped_attributes_count: 0,
        parent_span_is_remote: false,
        instrumentation_scope: opentelemetry::InstrumentationScope::builder("test")
            .with_version("1.0.0")
            .build(),
    }
}

#[tokio::test]
async fn test_public_api_trace_export() {
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
    
    // Export a trace using public API
    let span = create_test_span("test-span");
    library.export_trace(span).await.expect("Failed to export trace");
    
    // Wait for batch write
    sleep(Duration::from_secs(2)).await;
    
    // Flush to ensure all writes are complete
    library.flush().await.expect("Failed to flush");
    
    // Verify file was created
    let traces_dir = temp_dir.path().join("otlp/traces");
    let files: Vec<_> = std::fs::read_dir(&traces_dir)
        .unwrap()
        .filter_map(|e| e.ok())
        .collect();
    
    assert!(!files.is_empty(), "Expected at least one trace file to be created");
    
    // Verify file is readable as Arrow IPC
    let first_file = files[0].path();
    assert!(first_file.exists(), "Trace file should exist");
    assert!(first_file.extension().unwrap() == "arrow" || first_file.file_name().unwrap().to_string_lossy().contains("arrow"),
        "Trace file should have .arrow extension or contain 'arrow' in name");
    
    // Cleanup
    library.shutdown().await.unwrap();
}

#[tokio::test]
async fn test_public_api_multiple_traces_export() {
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

    let library = OtlpLibrary::new(config.clone()).await.unwrap();
    
    // Export multiple traces
    let spans = vec![
        create_test_span("span-1"),
        create_test_span("span-2"),
        create_test_span("span-3"),
    ];
    
    library.export_traces(spans).await.expect("Failed to export traces");
    
    // Wait for batch write
    sleep(Duration::from_secs(2)).await;
    
    // Flush to ensure all writes are complete
    library.flush().await.expect("Failed to flush");
    
    // Verify file was created
    let traces_dir = temp_dir.path().join("otlp/traces");
    let files: Vec<_> = std::fs::read_dir(&traces_dir)
        .unwrap()
        .filter_map(|e| e.ok())
        .collect();
    
    assert!(!files.is_empty(), "Expected at least one trace file to be created");
    
    library.shutdown().await.unwrap();
}

