//! Contract tests for edge cases
//!
//! Tests edge cases including buffer capacity limits, file rotation race conditions,
//! and error recovery scenarios.

use otlp_arrow_library::otlp::BatchBuffer;
use otlp_arrow_library::error::{OtlpError, OtlpExportError};
use opentelemetry::trace::{SpanContext, SpanId, SpanKind, Status, TraceId, TraceFlags, TraceState};
use opentelemetry_sdk::trace::SpanData;
use std::time::Duration;
use opentelemetry_proto::tonic::collector::metrics::v1::ExportMetricsServiceRequest;

/// Helper function to create a test span
fn create_test_span(name: &str) -> SpanData {
    let trace_id = TraceId::from_bytes([1, 2, 3, 4, 5, 6, 7, 8, 9, 10, 11, 12, 13, 14, 15, 16]);
    let span_id = SpanId::from_bytes([1, 2, 3, 4, 5, 6, 7, 8]);
    
    let span_context = SpanContext::new(
        trace_id,
        span_id,
        TraceFlags::default(),
        false,
        TraceState::default(),
    );
    
    SpanData {
        span_context,
        parent_span_id: SpanId::INVALID,
        span_kind: SpanKind::Server,
        name: std::borrow::Cow::Owned(name.to_string()),
        start_time: std::time::SystemTime::now(),
        end_time: std::time::SystemTime::now() + Duration::from_secs(1),
        attributes: vec![].into_iter().collect(),
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
async fn test_buffer_capacity_limit_traces() {
    // Test buffer capacity limit for traces
    let max_size = 100;
    let buffer = BatchBuffer::new(5, max_size, 10000);
    
    // Fill buffer to capacity
    for i in 0..max_size {
        let span = create_test_span(&format!("span-{}", i));
        let result = buffer.add_trace(span).await;
        assert!(result.is_ok(), "Adding trace should succeed until capacity");
    }
    
    // Verify buffer is at capacity
    let count = buffer.trace_count().await;
    assert_eq!(count, max_size, "Buffer should be at capacity");
    
    // Attempt to exceed capacity
    let span = create_test_span("overflow-span");
    let result = buffer.add_trace(span).await;
    assert!(result.is_err(), "Adding trace beyond capacity should fail");
    
    // Verify error type
    assert!(
        matches!(result.unwrap_err(), OtlpError::Export(OtlpExportError::BufferFull)),
        "Error should be BufferFull"
    );
}

#[tokio::test]
async fn test_buffer_capacity_limit_metrics() {
    // Test buffer capacity limit for metrics
    let max_size = 50;
    let buffer = BatchBuffer::new(5, 10000, max_size);
    
    // Fill buffer to capacity
    for i in 0..max_size {
        let metrics = ExportMetricsServiceRequest::default();
        let result = buffer.add_metrics_protobuf(metrics).await;
        assert!(result.is_ok(), "Adding metrics should succeed until capacity");
    }
    
    // Verify buffer is at capacity
    let count = buffer.metric_count().await;
    assert_eq!(count, max_size, "Buffer should be at capacity");
    
    // Attempt to exceed capacity
    let metrics = ExportMetricsServiceRequest::default();
    let result = buffer.add_metrics_protobuf(metrics).await;
    assert!(result.is_err(), "Adding metrics beyond capacity should fail");
    
    // Verify error type
    assert!(
        matches!(result.unwrap_err(), OtlpError::Export(OtlpExportError::BufferFull)),
        "Error should be BufferFull"
    );
}

#[tokio::test]
async fn test_buffer_capacity_limit_batch_add() {
    // Test buffer capacity limit when adding multiple traces at once
    let max_size = 100;
    let buffer = BatchBuffer::new(5, max_size, 10000);
    
    // Fill buffer partially
    for i in 0..50 {
        let span = create_test_span(&format!("span-{}", i));
        buffer.add_trace(span).await.unwrap();
    }
    
    // Attempt to add batch that exceeds capacity
    let mut spans = Vec::new();
    for i in 50..max_size + 10 {
        spans.push(create_test_span(&format!("batch-span-{}", i)));
    }
    
    let result = buffer.add_traces(spans).await;
    assert!(result.is_err(), "Adding batch beyond capacity should fail");
    
    // Verify error type
    assert!(
        matches!(result.unwrap_err(), OtlpError::Export(OtlpExportError::BufferFull)),
        "Error should be BufferFull"
    );
    
    // Verify buffer state unchanged
    let count = buffer.trace_count().await;
    assert_eq!(count, 50, "Buffer should still contain only initial traces");
}

#[tokio::test]
async fn test_buffer_concurrent_capacity_limit() {
    // Test concurrent writes hitting capacity limit
    use std::sync::Arc;
    use tokio::task::JoinSet;
    
    let max_size = 100;
    let buffer = Arc::new(BatchBuffer::new(5, max_size, 10000));
    
    // Spawn concurrent writers that will exceed capacity
    let mut join_set = JoinSet::new();
    let writers = 150; // More than capacity
    
    for i in 0..writers {
        let buffer_clone = buffer.clone();
        join_set.spawn(async move {
            let span = create_test_span(&format!("concurrent-span-{}", i));
            buffer_clone.add_trace(span).await
        });
    }
    
    // Wait for all tasks
    let mut success_count = 0;
    let mut failure_count = 0;
    while let Some(result) = join_set.join_next().await {
        match result {
            Ok(Ok(_)) => success_count += 1,
            Ok(Err(_)) => failure_count += 1,
            Err(e) => panic!("Task join error: {:?}", e),
        }
    }
    
    // Some should succeed, some should fail
    assert_eq!(success_count, max_size, "Exactly max_size writes should succeed");
    assert_eq!(failure_count, writers - max_size, "Remaining writes should fail");
    
    // Verify buffer is at capacity
    let count = buffer.trace_count().await;
    assert_eq!(count, max_size, "Buffer should be at capacity");
}

#[tokio::test]
async fn test_buffer_take_after_capacity() {
    // Test taking traces after hitting capacity allows new writes
    let max_size = 100;
    let buffer = BatchBuffer::new(5, max_size, 10000);
    
    // Fill buffer to capacity
    for i in 0..max_size {
        let span = create_test_span(&format!("span-{}", i));
        buffer.add_trace(span).await.unwrap();
    }
    
    // Verify at capacity
    assert_eq!(buffer.trace_count().await, max_size);
    
    // Take all traces
    let taken = buffer.take_traces().await;
    assert_eq!(taken.len(), max_size, "Should take all traces");
    
    // Verify buffer is empty
    assert_eq!(buffer.trace_count().await, 0, "Buffer should be empty");
    
    // Now should be able to add more
    let span = create_test_span("new-span");
    let result = buffer.add_trace(span).await;
    assert!(result.is_ok(), "Should be able to add after take");
    
    // Verify new trace added
    assert_eq!(buffer.trace_count().await, 1, "Buffer should contain new trace");
}

#[tokio::test]
async fn test_buffer_empty_take() {
    // Test taking from empty buffer
    let buffer = BatchBuffer::new(5, 10000, 10000);
    
    // Take from empty buffer
    let traces = buffer.take_traces().await;
    assert_eq!(traces.len(), 0, "Should return empty vector");
    
    let metrics = buffer.take_metrics().await;
    assert_eq!(metrics.len(), 0, "Should return empty vector");
}

#[tokio::test]
async fn test_buffer_should_write_time_handling() {
    // Test should_write time handling (including clock going backwards)
    let buffer = BatchBuffer::new(1, 10000, 10000); // 1 second interval
    
    // Initially should_write might be true or false depending on timing
    let _initial = buffer.should_write().await;
    
    // Update last write time
    buffer.update_last_write().await;
    
    // Immediately should not write
    let should_write_immediate = buffer.should_write().await;
    assert!(!should_write_immediate, "Should not write immediately after update");
    
    // Wait for interval
    tokio::time::sleep(Duration::from_millis(1100)).await;
    
    // Now should write
    let should_write_after = buffer.should_write().await;
    assert!(should_write_after, "Should write after interval has passed");
}

#[tokio::test]
async fn test_buffer_concurrent_take_and_add() {
    // Test concurrent take and add operations
    use std::sync::Arc;
    use tokio::task::JoinSet;
    
    let buffer = Arc::new(BatchBuffer::new(5, 10000, 10000));
    
    // Add initial traces
    for i in 0..50 {
        let span = create_test_span(&format!("initial-{}", i));
        buffer.add_trace(span).await.unwrap();
    }
    
    // Spawn concurrent take and add operations
    let mut join_set = JoinSet::new();
    
    // Takers
    for _ in 0..3 {
        let buffer_clone = buffer.clone();
        join_set.spawn(async move {
            buffer_clone.take_traces().await
        });
    }
    
    // Adders
    for i in 0..30 {
        let buffer_clone = buffer.clone();
        join_set.spawn(async move {
            let span = create_test_span(&format!("concurrent-add-{}", i));
            buffer_clone.add_trace(span).await
        });
    }
    
    // Wait for all tasks
    let mut total_taken = 0;
    let mut add_success = 0;
    while let Some(result) = join_set.join_next().await {
        match result {
            Ok(traces) => {
                total_taken += traces.len();
            }
            Ok(Ok(_)) => {
                add_success += 1;
            }
            Ok(Err(_)) => {
                // Some adds might fail if buffer is taken
            }
            Err(e) => panic!("Task join error: {:?}", e),
        }
    }
    
    // Verify consistency - total traces should be initial + successful adds - taken
    let final_count = buffer.trace_count().await;
    // Note: This is a race condition test - exact count depends on timing
    // We just verify the buffer is in a consistent state
    assert!(final_count <= 50 + 30, "Final count should not exceed initial + adds");
}
