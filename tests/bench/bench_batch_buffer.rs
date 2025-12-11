//! Performance benchmark for BatchBuffer throughput
//!
//! Measures BatchBuffer throughput under high concurrency to verify
//! that locking optimizations improve performance while maintaining correctness.

use criterion::{black_box, criterion_group, criterion_main, Criterion, BenchmarkId};
use otlp_arrow_library::otlp::BatchBuffer;
use opentelemetry::trace::{SpanContext, SpanId, SpanKind, Status, TraceId, TraceFlags, TraceState};
use opentelemetry_sdk::trace::SpanData;
use std::sync::Arc;
use std::time::Duration;
use tokio::runtime::Runtime;
use tokio::task::JoinSet;

/// Helper function to create a test span
fn create_test_span(i: usize) -> SpanData {
    let trace_id = TraceId::from_bytes([i as u8; 16]);
    let span_id = SpanId::from_bytes([i as u8; 8]);
    
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
        name: std::borrow::Cow::Owned(format!("bench-span-{}", i)),
        start_time: std::time::SystemTime::now(),
        end_time: std::time::SystemTime::now() + Duration::from_secs(1),
        attributes: vec![].into_iter().collect(),
        events: opentelemetry_sdk::trace::SpanEvents::default(),
        links: opentelemetry_sdk::trace::SpanLinks::default(),
        status: Status::Ok,
        dropped_attributes_count: 0,
        parent_span_is_remote: false,
        instrumentation_scope: opentelemetry::InstrumentationScope::builder("bench")
            .with_version("1.0.0")
            .build(),
    }
}

fn bench_batch_buffer_single_add(c: &mut Criterion) {
    let rt = Runtime::new().unwrap();
    let buffer = Arc::new(BatchBuffer::new(5, 100000, 100000));
    
    c.bench_function("batch_buffer_single_add", |b| {
        let buffer_clone = buffer.clone();
        b.to_async(&rt).iter(|| {
            let buffer = buffer_clone.clone();
            let span = create_test_span(0);
            async move {
                black_box(buffer.add_trace(span).await)
            }
        });
    });
}

fn bench_batch_buffer_batch_add(c: &mut Criterion) {
    let rt = Runtime::new().unwrap();
    let buffer = Arc::new(BatchBuffer::new(5, 100000, 100000));
    
    c.bench_function("batch_buffer_batch_add", |b| {
        let buffer_clone = buffer.clone();
        b.to_async(&rt).iter(|| {
            let buffer = buffer_clone.clone();
            let spans: Vec<_> = (0..100).map(|i| create_test_span(i)).collect();
            async move {
                black_box(buffer.add_traces(spans).await)
            }
        });
    });
}

fn bench_batch_buffer_concurrent_throughput(c: &mut Criterion) {
    let rt = Runtime::new().unwrap();
    
    let mut group = c.benchmark_group("batch_buffer_concurrent");
    for concurrency in [10, 50, 100, 500].iter() {
        group.bench_with_input(
            BenchmarkId::from_parameter(concurrency),
            concurrency,
            |b, &concurrency| {
                let buffer = Arc::new(BatchBuffer::new(5, 100000, 100000));
                b.to_async(&rt).iter(|| {
                    let buffer_clone = buffer.clone();
                    async move {
                        let mut join_set = JoinSet::new();
                        for i in 0..concurrency {
                            let buffer = buffer_clone.clone();
                            let span = create_test_span(i);
                            join_set.spawn(async move {
                                buffer.add_trace(span).await
                            });
                        }
                        
                        while let Some(result) = join_set.join_next().await {
                            black_box(result);
                        }
                    }
                });
            },
        );
    }
    group.finish();
}

fn bench_batch_buffer_mixed_operations(c: &mut Criterion) {
    let rt = Runtime::new().unwrap();
    let buffer = Arc::new(BatchBuffer::new(5, 100000, 100000));
    
    c.bench_function("batch_buffer_mixed_read_write", |b| {
        let buffer_clone = buffer.clone();
        b.to_async(&rt).iter(|| {
            let buffer = buffer_clone.clone();
            async move {
                // Add some traces
                for i in 0..50 {
                    let span = create_test_span(i);
                    let _ = buffer.add_trace(span).await;
                }
                
                // Read count
                let _count = buffer.trace_count().await;
                
                // Add more traces
                for i in 50..100 {
                    let span = create_test_span(i);
                    let _ = buffer.add_trace(span).await;
                }
                
                // Read count again
                let _count = buffer.trace_count().await;
                
                black_box(())
            }
        });
    });
}

criterion_group!(
    benches,
    bench_batch_buffer_single_add,
    bench_batch_buffer_batch_add,
    bench_batch_buffer_concurrent_throughput,
    bench_batch_buffer_mixed_operations
);
criterion_main!(benches);
