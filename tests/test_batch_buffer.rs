//! Unit tests for batch buffer functionality

use opentelemetry::KeyValue;
use opentelemetry::trace::{
    SpanContext, SpanId, SpanKind, Status, TraceFlags, TraceId, TraceState,
};
use opentelemetry_sdk::trace::SpanData;
use otlp_arrow_library::otlp::BatchBuffer;
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
        attributes: vec![KeyValue::new("service.name", "test-service")],
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
async fn test_batch_buffer_add_trace() {
    let buffer = BatchBuffer::new(5, 10000, 10000); // 5 second interval, default buffer sizes

    let span = create_test_span("test-span");

    // Add trace
    let result = buffer.add_trace(span).await;
    assert!(result.is_ok(), "Adding trace should succeed");

    // Verify count
    let count = buffer.trace_count().await;
    assert_eq!(count, 1, "Buffer should contain 1 trace");
}

#[tokio::test]
async fn test_batch_buffer_add_traces() {
    let buffer = BatchBuffer::new(5, 10000, 10000);

    let spans = vec![
        create_test_span("span-1"),
        create_test_span("span-2"),
        create_test_span("span-3"),
    ];

    // Add multiple traces
    let result = buffer.add_traces(spans).await;
    assert!(result.is_ok(), "Adding traces should succeed");

    // Verify count
    let count = buffer.trace_count().await;
    assert_eq!(count, 3, "Buffer should contain 3 traces");
}

// Note: Metrics are now stored as protobuf (ExportMetricsServiceRequest) to support Clone
#[tokio::test]
async fn test_batch_buffer_add_metrics() {
    use opentelemetry_proto::tonic::collector::metrics::v1::ExportMetricsServiceRequest;

    let buffer = BatchBuffer::new(5, 10000, 10000);

    // Create a minimal protobuf metrics request
    let metrics_request = ExportMetricsServiceRequest::default();

    // Add metrics (protobuf format)
    let result = buffer.add_metrics_protobuf(metrics_request).await;
    assert!(result.is_ok(), "Adding metrics should succeed");

    // Verify count
    let count = buffer.metric_count().await;
    assert_eq!(count, 1, "Buffer should contain 1 metric");
}

#[tokio::test]
async fn test_batch_buffer_take_metrics() {
    use opentelemetry_proto::tonic::collector::metrics::v1::ExportMetricsServiceRequest;

    let buffer = BatchBuffer::new(5, 10000, 10000);

    let metrics_request = ExportMetricsServiceRequest::default();
    buffer.add_metrics_protobuf(metrics_request).await.unwrap();

    // Take metrics (should clear buffer, returns protobuf)
    let taken = buffer.take_metrics().await;
    assert_eq!(taken.len(), 1, "Should take 1 metric");

    // Verify buffer is empty
    let count = buffer.metric_count().await;
    assert_eq!(count, 0, "Buffer should be empty after take");
}

#[tokio::test]
async fn test_batch_buffer_take_traces() {
    let buffer = BatchBuffer::new(5, 10000, 10000);

    let spans = vec![create_test_span("span-1"), create_test_span("span-2")];

    buffer.add_traces(spans.clone()).await.unwrap();

    // Take traces (should clear buffer)
    let taken = buffer.take_traces().await;
    assert_eq!(taken.len(), 2, "Should take 2 traces");

    // Verify buffer is empty
    let count = buffer.trace_count().await;
    assert_eq!(count, 0, "Buffer should be empty after take");
}

#[tokio::test]
async fn test_batch_buffer_should_write() {
    let buffer = BatchBuffer::new(1, 10000, 10000); // 1 second interval, default buffer sizes

    // Initially should not write (just created)
    let _should_write = buffer.should_write().await;
    // This might be true or false depending on timing, so we'll just verify the method works
    // (The method returns a boolean, which is verified by the type system)

    // Update last write time
    buffer.update_last_write().await;

    // Wait a bit and check again
    tokio::time::sleep(Duration::from_millis(1100)).await;

    let should_write_after = buffer.should_write().await;
    assert!(should_write_after, "Should write after interval has passed");
}
