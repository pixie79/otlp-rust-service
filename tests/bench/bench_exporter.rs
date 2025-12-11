//! Performance benchmark for exporter implementations
//!
//! Measures exporter throughput, latency, and memory allocations to verify
//! that optimizations improve performance without increasing resource usage.

use criterion::{black_box, criterion_group, criterion_main, Criterion, BenchmarkId};
use otlp_arrow_library::{ConfigBuilder, OtlpLibrary};
use opentelemetry_sdk::trace::SpanData;
use opentelemetry::trace::{SpanContext, SpanId, SpanKind, Status, TraceId, TraceFlags, TraceState};
use opentelemetry_proto::tonic::collector::metrics::v1::ExportMetricsServiceRequest;
use tempfile::TempDir;
use tokio::runtime::Runtime;
use std::sync::Arc;

fn create_test_span(i: usize) -> SpanData {
    let trace_id = TraceId::from_bytes([i as u8; 16]);
    let span_id = SpanId::from_bytes([i as u8; 8]);
    let span_context = SpanContext::new(trace_id, span_id, TraceFlags::default(), false, TraceState::default());

    SpanData {
        span_context,
        parent_span_id: SpanId::INVALID,
        span_kind: SpanKind::Internal,
        name: std::borrow::Cow::Owned(format!("bench-span-{}", i)),
        start_time: std::time::SystemTime::now(),
        end_time: std::time::SystemTime::now(),
        attributes: vec![].into_iter().collect(),
        events: opentelemetry_sdk::trace::SpanEvents::default(),
        links: opentelemetry_sdk::trace::SpanLinks::default(),
        status: Status::Ok,
        dropped_attributes_count: 0,
        parent_span_is_remote: false,
        instrumentation_scope: opentelemetry::InstrumentationScope::builder("bench")
            .build(),
    }
}

fn bench_exporter_trace_throughput(c: &mut Criterion) {
    let rt = Runtime::new().unwrap();
    
    c.bench_function("exporter_trace_single", |b| {
        let temp_dir = TempDir::new().unwrap();
        let config = ConfigBuilder::new()
            .output_dir(temp_dir.path())
            .write_interval_secs(1)
            .build()
            .unwrap();
        
        let library = rt.block_on(OtlpLibrary::new(config)).unwrap();
        let span = create_test_span(0);
        
        b.to_async(&rt).iter(|| {
            let library_clone = library.clone();
            let span_clone = SpanData {
                span_context: span.span_context.clone(),
                parent_span_id: span.parent_span_id,
                span_kind: span.span_kind,
                name: span.name.clone(),
                start_time: span.start_time,
                end_time: span.end_time,
                attributes: span.attributes.clone(),
                events: span.events.clone(),
                links: span.links.clone(),
                status: span.status,
                dropped_attributes_count: span.dropped_attributes_count,
                parent_span_is_remote: span.parent_span_is_remote,
                instrumentation_scope: span.instrumentation_scope.clone(),
            };
            async move {
                black_box(library_clone.export_trace(span_clone).await)
            }
        });
    });
}

fn bench_exporter_trace_batch(c: &mut Criterion) {
    let rt = Runtime::new().unwrap();
    
    let mut group = c.benchmark_group("exporter_trace_batch");
    for batch_size in [10, 100, 1000].iter() {
        group.bench_with_input(
            BenchmarkId::from_parameter(batch_size),
            batch_size,
            |b, &batch_size| {
                let temp_dir = TempDir::new().unwrap();
                let config = ConfigBuilder::new()
                    .output_dir(temp_dir.path())
                    .write_interval_secs(1)
                    .build()
                    .unwrap();
                
                let library = rt.block_on(OtlpLibrary::new(config)).unwrap();
                let spans: Vec<_> = (0..batch_size).map(|i| create_test_span(i)).collect();
                
                b.to_async(&rt).iter(|| {
                    let library_clone = library.clone();
                    let spans_clone: Vec<_> = spans.iter().map(|s| {
                        SpanData {
                            span_context: s.span_context.clone(),
                            parent_span_id: s.parent_span_id,
                            span_kind: s.span_kind,
                            name: s.name.clone(),
                            start_time: s.start_time,
                            end_time: s.end_time,
                            attributes: s.attributes.clone(),
                            events: s.events.clone(),
                            links: s.links.clone(),
                            status: s.status,
                            dropped_attributes_count: s.dropped_attributes_count,
                            parent_span_is_remote: s.parent_span_is_remote,
                            instrumentation_scope: s.instrumentation_scope.clone(),
                        }
                    }).collect();
                    async move {
                        black_box(library_clone.export_traces(spans_clone).await)
                    }
                });
            },
        );
    }
    group.finish();
}

fn bench_exporter_metrics_throughput(c: &mut Criterion) {
    let rt = Runtime::new().unwrap();
    
    c.bench_function("exporter_metrics_single", |b| {
        let temp_dir = TempDir::new().unwrap();
        let config = ConfigBuilder::new()
            .output_dir(temp_dir.path())
            .write_interval_secs(1)
            .build()
            .unwrap();
        
        let library = rt.block_on(OtlpLibrary::new(config)).unwrap();
        let metrics = ExportMetricsServiceRequest::default();
        
        b.to_async(&rt).iter(|| {
            let library_clone = library.clone();
            let metrics_clone = metrics.clone();
            async move {
                black_box(library_clone.export_metrics_ref(&metrics_clone).await)
            }
        });
    });
}

fn bench_exporter_concurrent_throughput(c: &mut Criterion) {
    let rt = Runtime::new().unwrap();
    
    let mut group = c.benchmark_group("exporter_concurrent");
    for concurrency in [10, 50, 100].iter() {
        group.bench_with_input(
            BenchmarkId::from_parameter(concurrency),
            concurrency,
            |b, &concurrency| {
                let temp_dir = TempDir::new().unwrap();
                let config = ConfigBuilder::new()
                    .output_dir(temp_dir.path())
                    .write_interval_secs(1)
                    .build()
                    .unwrap();
                
                let library = Arc::new(rt.block_on(OtlpLibrary::new(config)).unwrap());
                
                b.to_async(&rt).iter(|| {
                    let library_clone = library.clone();
                    async move {
                        use tokio::task::JoinSet;
                        let mut join_set = JoinSet::new();
                        
                        for i in 0..concurrency {
                            let library = library_clone.clone();
                            let span = create_test_span(i);
                            join_set.spawn(async move {
                                library.export_trace(span).await
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

criterion_group!(
    benches,
    bench_exporter_trace_throughput,
    bench_exporter_trace_batch,
    bench_exporter_metrics_throughput,
    bench_exporter_concurrent_throughput
);
criterion_main!(benches);
