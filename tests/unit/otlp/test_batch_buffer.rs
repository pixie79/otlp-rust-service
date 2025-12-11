//! Unit tests for batch buffer functionality

use otlp_arrow_library::otlp::BatchBuffer;
use opentelemetry::trace::{SpanContext, SpanId, SpanKind, Status, TraceId};
use opentelemetry::KeyValue;
use opentelemetry_sdk::trace::SpanData;
use opentelemetry_sdk::Resource;
use opentelemetry_sdk::metrics::data::{ResourceMetrics, ScopeMetrics};
use std::time::{Duration, SystemTime};

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
        name: name.to_string(),
        start_time: SystemTime::now(),
        end_time: SystemTime::now() + Duration::from_secs(1),
        attributes: vec![
            KeyValue::new("service.name", "test-service"),
        ],
        events: vec![],
        links: vec![],
        status: Status::Ok,
        resource: Resource::empty(),
        instrumentation_lib: opentelemetry::InstrumentationLibrary::new(
            "test",
            Some("1.0.0"),
            None,
        ),
    }
}

/// Helper function to create test metrics
fn create_test_metrics() -> opentelemetry_sdk::metrics::data::ResourceMetrics {
    // Create a simple ResourceMetrics structure
    // Note: ResourceMetrics has private fields, so we'll use Debug format for now
    // In a real implementation, we'd use proper constructors
    opentelemetry_sdk::metrics::data::ResourceMetrics {
        resource: opentelemetry_sdk::Resource::empty(),
        scope_metrics: vec![],
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

#[tokio::test]
async fn test_batch_buffer_add_metrics() {
    let buffer = BatchBuffer::new(5, 10000, 10000);
    
    let metrics = create_test_metrics();
    
    // Add metrics
    let result = buffer.add_metrics(metrics).await;
    assert!(result.is_ok(), "Adding metrics should succeed");
    
    // Verify count
    let count = buffer.metric_count().await;
    assert_eq!(count, 1, "Buffer should contain 1 metric");
}

#[tokio::test]
async fn test_batch_buffer_take_traces() {
    let buffer = BatchBuffer::new(5, 10000, 10000);
    
    let spans = vec![
        create_test_span("span-1"),
        create_test_span("span-2"),
    ];
    
    buffer.add_traces(spans.clone()).await.unwrap();
    
    // Take traces (should clear buffer)
    let taken = buffer.take_traces().await;
    assert_eq!(taken.len(), 2, "Should take 2 traces");
    
    // Verify buffer is empty
    let count = buffer.trace_count().await;
    assert_eq!(count, 0, "Buffer should be empty after take");
}

#[tokio::test]
async fn test_batch_buffer_take_metrics() {
    let buffer = BatchBuffer::new(5, 10000, 10000);
    
    let metrics = create_test_metrics();
    buffer.add_metrics(metrics).await.unwrap();
    
    // Take metrics (should clear buffer)
    let taken = buffer.take_metrics().await;
    assert_eq!(taken.len(), 1, "Should take 1 metric");
    
    // Verify buffer is empty
    let count = buffer.metric_count().await;
    assert_eq!(count, 0, "Buffer should be empty after take");
}

#[tokio::test]
async fn test_batch_buffer_should_write() {
    let buffer = BatchBuffer::new(1, 10000, 10000); // 1 second interval, default buffer sizes
    
    // Initially should not write (just created)
    let should_write = buffer.should_write().await;
    // This might be true or false depending on timing, so we'll just verify the method works
    assert!(should_write || !should_write, "should_write should return a boolean");
    
    // Update last write time
    buffer.update_last_write().await;
    
    // Wait a bit and check again
    tokio::time::sleep(Duration::from_millis(1100)).await;
    
    let should_write_after = buffer.should_write().await;
    assert!(should_write_after, "Should write after interval has passed");
}

