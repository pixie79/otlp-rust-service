//! Integration test for file writing with default output directory

use otlp_arrow_library::{Config, OtlpLibrary};
use opentelemetry::trace::{SpanContext, SpanId, SpanKind, Status, TraceId, TraceFlags, TraceState};
use opentelemetry::KeyValue;
use opentelemetry_sdk::trace::SpanData;
use opentelemetry_sdk::Resource;
use std::path::PathBuf;
use std::time::{Duration, SystemTime};
use tempfile::TempDir;

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
        name: std::borrow::Cow::Borrowed(name),
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
async fn test_file_writing_default_output_dir() {
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
    let library = OtlpLibrary::new(config).await.unwrap();

    // Export a trace
    let span = create_test_span("test-span");
    library.export_trace(span).await.unwrap();

    // Flush to ensure writes complete
    library.flush().await.unwrap();

    // Verify directory structure was created
    let traces_dir = temp_dir.path().join("otlp/traces");
    let metrics_dir = temp_dir.path().join("otlp/metrics");
    
    assert!(traces_dir.exists(), "Traces directory should exist");
    assert!(metrics_dir.exists(), "Metrics directory should exist");

    // Verify at least one arrow file was created in traces directory
    let files: Vec<_> = std::fs::read_dir(&traces_dir)
        .unwrap()
        .filter_map(|entry| entry.ok())
        .filter(|entry| {
            entry.path().extension()
                .and_then(|ext| ext.to_str())
                .map(|ext| ext == "arrow")
                .unwrap_or(false)
        })
        .collect();
    
    assert!(!files.is_empty(), "At least one arrow file should be created in traces directory");
    
    // Verify the file is readable as Arrow IPC
    use arrow::ipc::reader::StreamReader;
    use std::fs::File;
    
    let arrow_file = File::open(&files[0].path()).unwrap();
    let reader = StreamReader::try_new(arrow_file, None).unwrap();
    
    let mut batch_count = 0;
    for batch_result in reader {
        let batch = batch_result.unwrap();
        batch_count += 1;
        
        // Verify schema has expected fields
        let schema = batch.schema();
        assert!(schema.field_with_name("trace_id").is_ok(), "Schema should have trace_id field");
        assert!(schema.field_with_name("span_id").is_ok(), "Schema should have span_id field");
        assert!(schema.field_with_name("name").is_ok(), "Schema should have name field");
    }
    
    assert!(batch_count > 0, "Should have at least one batch");
    
    // Cleanup
    library.shutdown().await.unwrap();
}
