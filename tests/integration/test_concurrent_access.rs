//! Integration tests for concurrent access scenarios
//!
//! Tests BatchBuffer and other components under high concurrency to ensure
//! data integrity and correct behavior.

use otlp_arrow_library::otlp::BatchBuffer;
use opentelemetry::trace::{SpanContext, SpanId, SpanKind, Status, TraceId, TraceFlags, TraceState};
use opentelemetry_sdk::trace::SpanData;
use std::sync::Arc;
use std::time::Duration;
use tokio::task::JoinSet;

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
async fn test_batch_buffer_high_concurrency() {
    // Test BatchBuffer under high concurrency (1000 concurrent writers)
    let buffer = Arc::new(BatchBuffer::new(5, 20000, 20000));
    let concurrency_level = 1000;
    
    // Spawn concurrent tasks using JoinSet
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
async fn test_batch_buffer_concurrent_read_write() {
    // Test concurrent reads and writes to BatchBuffer
    let buffer = Arc::new(BatchBuffer::new(5, 10000, 10000));
    
    // Add initial traces
    for i in 0..50 {
        let span = create_test_span(&format!("initial-{}", i));
        buffer.add_trace(span).await.unwrap();
    }
    
    // Spawn concurrent readers and writers
    let mut join_set = JoinSet::new();
    
    // Writers
    for i in 0..100 {
        let buffer_clone = buffer.clone();
        join_set.spawn(async move {
            let span = create_test_span(&format!("write-{}", i));
            buffer_clone.add_trace(span).await
        });
    }
    
    // Readers (trace_count)
    for _ in 0..20 {
        let buffer_clone = buffer.clone();
        join_set.spawn(async move {
            buffer_clone.trace_count().await
        });
    }
    
    // Wait for all tasks
    let mut write_success = 0;
    while let Some(result) = join_set.join_next().await {
        match result {
            Ok(Ok(_)) => write_success += 1,
            Ok(Ok(count)) if count > 0 => {
                // Reader result - just verify it's a valid count
                assert!(count >= 50, "Count should be at least initial traces");
            }
            Ok(Err(_)) => panic!("Write operation failed"),
            Err(e) => panic!("Task join error: {:?}", e),
        }
    }
    
    // Validate writers succeeded
    assert_eq!(write_success, 100, "All writes should succeed");
    
    // Validate final count
    let final_count = buffer.trace_count().await;
    assert_eq!(final_count, 150, "Buffer should contain 50 initial + 100 new traces");
}

#[tokio::test]
async fn test_batch_buffer_concurrent_take_operations() {
    // Test concurrent take operations (should be safe with locks)
    let buffer = Arc::new(BatchBuffer::new(5, 10000, 10000));
    
    // Add traces
    for i in 0..100 {
        let span = create_test_span(&format!("span-{}", i));
        buffer.add_trace(span).await.unwrap();
    }
    
    // Spawn concurrent take operations
    let mut join_set = JoinSet::new();
    
    for _ in 0..5 {
        let buffer_clone = buffer.clone();
        join_set.spawn(async move {
            buffer_clone.take_traces().await
        });
    }
    
    // Wait for all tasks
    let mut total_taken = 0;
    while let Some(result) = join_set.join_next().await {
        match result {
            Ok(traces) => {
                total_taken += traces.len();
            }
            Err(e) => panic!("Task join error: {:?}", e),
        }
    }
    
    // Only one take should get all traces (others get empty)
    assert_eq!(total_taken, 100, "All traces should be taken exactly once");
    
    // Buffer should be empty
    let final_count = buffer.trace_count().await;
    assert_eq!(final_count, 0, "Buffer should be empty after take");
}
