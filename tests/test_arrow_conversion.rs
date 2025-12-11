//! Unit tests for Arrow IPC conversion functions

use opentelemetry::KeyValue;
use opentelemetry::trace::{
    SpanContext, SpanId, SpanKind, Status, TraceFlags, TraceId, TraceState,
};
use opentelemetry_sdk::trace::SpanData;
use std::time::{Duration, SystemTime};

/// Helper function to create a test span
fn create_test_span(name: &str) -> SpanData {
    let trace_id = TraceId::from_bytes([1, 2, 3, 4, 5, 6, 7, 8, 9, 10, 11, 12, 13, 14, 15, 16]);
    let span_id = SpanId::from_bytes([1, 2, 3, 4, 5, 6, 7, 8]);
    let parent_span_id = SpanId::from_bytes([9, 10, 11, 12, 13, 14, 15, 16]);

    let span_context = SpanContext::new(
        trace_id,
        span_id,
        TraceFlags::default(),
        false,
        TraceState::default(),
    );

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
async fn test_arrow_ipc_conversion_traces() {
    // Create test spans
    let spans = vec![
        create_test_span("test-span-1"),
        create_test_span("test-span-2"),
    ];

    // Import the exporter to test conversion
    use otlp_arrow_library::ConfigBuilder;
    use otlp_arrow_library::otlp::OtlpFileExporter;
    use tempfile::TempDir;

    // Create a temporary directory for testing
    let temp_dir = TempDir::new().unwrap();
    let config = ConfigBuilder::new()
        .output_dir(temp_dir.path())
        .write_interval_secs(5)
        .build()
        .unwrap();

    // Create exporter
    let exporter = OtlpFileExporter::new(&config).unwrap();

    // Export traces (this internally uses the conversion function)
    let result = exporter.export_traces(spans.clone()).await;
    assert!(
        result.is_ok(),
        "Failed to export traces: {:?}",
        result.err()
    );

    // Flush to ensure all writes are completed
    let flush_result = exporter.flush().await;
    assert!(
        flush_result.is_ok(),
        "Failed to flush exporter: {:?}",
        flush_result.err()
    );

    // Verify file was created
    let traces_dir = temp_dir.path().join("otlp/traces");
    assert!(traces_dir.exists(), "Traces directory should exist");

    // Check that at least one arrow file was created
    let files: Vec<_> = std::fs::read_dir(&traces_dir)
        .unwrap()
        .filter_map(|entry| entry.ok())
        .filter(|entry| {
            entry
                .path()
                .extension()
                .and_then(|ext| ext.to_str())
                .map(|ext| ext == "arrows")
                .unwrap_or(false)
        })
        .collect();

    assert!(
        !files.is_empty(),
        "At least one arrow file should be created"
    );

    // Verify the file is readable as Arrow IPC
    use arrow::ipc::reader::StreamReader;
    use std::fs::File;

    let arrow_file = File::open(files[0].path()).unwrap();
    let reader = StreamReader::try_new(arrow_file, None).unwrap();

    let mut batch_count = 0;
    for batch_result in reader {
        let batch = batch_result.unwrap();
        batch_count += 1;

        // Verify schema has expected fields
        let schema = batch.schema();
        assert!(
            schema.field_with_name("trace_id").is_ok(),
            "Schema should have trace_id field"
        );
        assert!(
            schema.field_with_name("span_id").is_ok(),
            "Schema should have span_id field"
        );
        assert!(
            schema.field_with_name("name").is_ok(),
            "Schema should have name field"
        );

        // Verify we have the expected number of rows
        assert_eq!(
            batch.num_rows(),
            spans.len(),
            "Batch should have correct number of rows"
        );
    }

    assert!(batch_count > 0, "Should have at least one batch");
}

#[tokio::test]
async fn test_arrow_ipc_conversion_empty_traces() {
    use otlp_arrow_library::ConfigBuilder;
    use otlp_arrow_library::otlp::OtlpFileExporter;
    use tempfile::TempDir;

    let temp_dir = TempDir::new().unwrap();
    let config = ConfigBuilder::new()
        .output_dir(temp_dir.path())
        .write_interval_secs(5)
        .build()
        .unwrap();

    let exporter = OtlpFileExporter::new(&config).unwrap();

    // Export empty traces should not fail
    let result = exporter.export_traces(vec![]).await;
    assert!(result.is_ok(), "Exporting empty traces should succeed");
}
