//! Unit tests for concurrent BatchBuffer access
//!
//! Tests concurrent access scenarios to ensure data integrity and correct behavior
//! under high concurrency.

use otlp_arrow_library::otlp::BatchBuffer;
use opentelemetry::trace::{SpanContext, SpanId, SpanKind, Status, TraceId};
use opentelemetry::KeyValue;
use opentelemetry_sdk::trace::SpanData;
use opentelemetry_sdk::Resource;
use std::sync::Arc;
use std::time::Duration;

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
        start_time: std::time::SystemTime::now(),
        end_time: std::time::SystemTime::now() + Duration::from_secs(1),
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

#[tokio::test]
async fn test_concurrent_batch_buffer_writers() {
    // Test multiple concurrent writers accessing BatchBuffer
    let buffer = Arc::new(BatchBuffer::new(5, 10000, 10000));
    let concurrency_level = 100;
    
    // Spawn concurrent tasks
    let mut handles = Vec::new();
    for i in 0..concurrency_level {
        let buffer_clone = buffer.clone();
        let handle = tokio::spawn(async move {
            let span = create_test_span(&format!("span-{}", i));
            buffer_clone.add_trace(span).await
        });
        handles.push(handle);
    }
    
    // Wait for all tasks
    let results: Vec<_> = futures::future::join_all(handles).await
        .into_iter()
        .map(|r| r.unwrap())
        .collect();
    
    // Validate all operations succeeded
    assert!(results.iter().all(|r| r.is_ok()), "All concurrent writes should succeed");
    
    // Validate final state consistency
    let count = buffer.trace_count().await;
    assert_eq!(count, concurrency_level, "Buffer should contain all traces");
}

#[tokio::test]
async fn test_concurrent_batch_buffer_read_write() {
    // Test concurrent reads and writes
    let buffer = Arc::new(BatchBuffer::new(5, 10000, 10000));
    
    // Add some initial traces
    for i in 0..10 {
        let span = create_test_span(&format!("initial-{}", i));
        buffer.add_trace(span).await.unwrap();
    }
    
    // Spawn concurrent readers and writers
    let mut handles = Vec::new();
    
    // Writers
    for i in 0..50 {
        let buffer_clone = buffer.clone();
        let handle = tokio::spawn(async move {
            let span = create_test_span(&format!("write-{}", i));
            buffer_clone.add_trace(span).await
        });
        handles.push(handle);
    }
    
    // Readers (trace_count)
    for _ in 0..10 {
        let buffer_clone = buffer.clone();
        let handle = tokio::spawn(async move {
            buffer_clone.trace_count().await
        });
        handles.push(handle);
    }
    
    // Wait for all tasks
    let results: Vec<_> = futures::future::join_all(handles).await
        .into_iter()
        .map(|r| r.unwrap())
        .collect();
    
    // Validate writers succeeded
    let write_results: Vec<_> = results.iter().take(50).collect();
    assert!(write_results.iter().all(|r| matches!(r, Ok(_))), "All writes should succeed");
    
    // Validate final count
    let final_count = buffer.trace_count().await;
    assert_eq!(final_count, 60, "Buffer should contain 10 initial + 50 new traces");
}

#[tokio::test]
async fn test_concurrent_batch_buffer_high_concurrency() {
    // Test with high concurrency level (1000 tasks)
    let buffer = Arc::new(BatchBuffer::new(5, 20000, 20000)); // Larger buffer for high concurrency
    let concurrency_level = 1000;
    
    // Spawn concurrent tasks using JoinSet for better control
    use tokio::task::JoinSet;
    let mut join_set = JoinSet::new();
    
    for i in 0..concurrency_level {
        let buffer_clone = buffer.clone();
        join_set.spawn(async move {
            let span = create_test_span(&format!("high-concurrency-{}", i));
            buffer_clone.add_trace(span).await
        });
    }
    
    // Wait for all tasks and collect results
    let mut success_count = 0;
    while let Some(result) = join_set.join_next().await {
        match result {
            Ok(Ok(_)) => success_count += 1,
            Ok(Err(_)) => panic!("Write operation failed"),
            Err(e) => panic!("Task join error: {:?}", e),
        }
    }
    
    // Validate all operations succeeded
    assert_eq!(success_count, concurrency_level, "All concurrent writes should succeed");
    
    // Validate final state consistency
    let count = buffer.trace_count().await;
    assert_eq!(count, concurrency_level, "Buffer should contain all traces");
}

#[tokio::test]
async fn test_concurrent_batch_buffer_metrics() {
    // Test concurrent metric writes
    use opentelemetry_proto::tonic::collector::metrics::v1::ExportMetricsServiceRequest;
    
    let buffer = Arc::new(BatchBuffer::new(5, 10000, 10000));
    let concurrency_level = 100;
    
    // Spawn concurrent metric writers
    let mut handles = Vec::new();
    for i in 0..concurrency_level {
        let buffer_clone = buffer.clone();
        let handle = tokio::spawn(async move {
            let metrics = ExportMetricsServiceRequest::default();
            buffer_clone.add_metrics_protobuf(metrics).await
        });
        handles.push(handle);
    }
    
    // Wait for all tasks
    let results: Vec<_> = futures::future::join_all(handles).await
        .into_iter()
        .map(|r| r.unwrap())
        .collect();
    
    // Validate all operations succeeded
    assert!(results.iter().all(|r| r.is_ok()), "All concurrent metric writes should succeed");
    
    // Validate final state consistency
    let count = buffer.metric_count().await;
    assert_eq!(count, concurrency_level, "Buffer should contain all metrics");
}

#[tokio::test]
async fn test_concurrent_batch_buffer_mixed_operations() {
    // Test mixed concurrent operations (add, take, count)
    let buffer = Arc::new(BatchBuffer::new(5, 10000, 10000));
    
    // Add initial traces
    for i in 0..20 {
        let span = create_test_span(&format!("initial-{}", i));
        buffer.add_trace(span).await.unwrap();
    }
    
    // Spawn mixed concurrent operations
    let mut handles = Vec::new();
    
    // Writers
    for i in 0..30 {
        let buffer_clone = buffer.clone();
        let handle = tokio::spawn(async move {
            let span = create_test_span(&format!("write-{}", i));
            buffer_clone.add_trace(span).await
        });
        handles.push(handle);
    }
    
    // Readers (count)
    for _ in 0..10 {
        let buffer_clone = buffer.clone();
        let handle = tokio::spawn(async move {
            buffer_clone.trace_count().await
        });
        handles.push(handle);
    }
    
    // Wait for all tasks
    let _results: Vec<_> = futures::future::join_all(handles).await
        .into_iter()
        .map(|r| r.unwrap())
        .collect();
    
    // Validate final state - should have 20 initial + 30 new = 50 traces
    let final_count = buffer.trace_count().await;
    assert_eq!(final_count, 50, "Buffer should contain all traces after mixed operations");
}
