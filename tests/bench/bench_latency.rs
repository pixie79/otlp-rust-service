//! Performance benchmark for latency

use criterion::{black_box, criterion_group, criterion_main, Criterion};
use otlp_arrow_library::{ConfigBuilder, OtlpLibrary};
use opentelemetry_sdk::trace::SpanData;
use opentelemetry::trace::{SpanContext, SpanId, SpanKind, Status, TraceId, TraceFlags, TraceState};
use std::time::Instant;
use tempfile::TempDir;
use tokio::runtime::Runtime;

fn create_test_span(i: usize) -> SpanData {
    let trace_id = TraceId::from_bytes([i as u8; 16]);
    let span_id = SpanId::from_bytes([i as u8; 8]);
    let span_context = SpanContext::new(trace_id, span_id, TraceFlags::default(), false, TraceState::default());

    SpanData {
        span_context,
        parent_span_id: SpanId::INVALID,
        span_kind: SpanKind::Internal,
        name: std::borrow::Cow::Owned(format!("latency-span-{}", i)),
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

fn bench_latency(c: &mut Criterion) {
    let rt = Runtime::new().unwrap();
    
    c.bench_function("export_trace_latency", |b| {
        let temp_dir = TempDir::new().unwrap();
        let config = ConfigBuilder::new()
            .output_dir(temp_dir.path())
            .write_interval_secs(1)
            .build()
            .unwrap();
        
        let library = rt.block_on(OtlpLibrary::new(config)).unwrap();
        let span = create_test_span(0);
        
        b.to_async(&rt).iter(|| {
            let span_clone = SpanData {
                span_context: span.span_context.clone(),
                parent_span_id: span.parent_span_id,
                span_kind: span.span_kind.clone(),
                name: span.name.clone(),
                start_time: std::time::SystemTime::now(),
                end_time: std::time::SystemTime::now(),
                attributes: span.attributes.clone(),
                events: span.events.clone(),
                links: span.links.clone(),
                status: span.status.clone(),
                dropped_attributes_count: span.dropped_attributes_count,
                parent_span_is_remote: span.parent_span_is_remote,
                instrumentation_scope: span.instrumentation_scope.clone(),
            };
            async {
                let start = Instant::now();
                library.export_trace(black_box(span_clone)).await.unwrap();
                let elapsed = start.elapsed();
                black_box(elapsed);
            }
        });
    });

    c.bench_function("flush_latency", |b| {
        let temp_dir = TempDir::new().unwrap();
        let config = ConfigBuilder::new()
            .output_dir(temp_dir.path())
            .write_interval_secs(1)
            .build()
            .unwrap();
        
        let library = rt.block_on(OtlpLibrary::new(config)).unwrap();
        
        b.to_async(&rt).iter(|| async {
            let start = Instant::now();
            library.flush().await.unwrap();
            let elapsed = start.elapsed();
            black_box(elapsed);
        });
    });
}

criterion_group!(benches, bench_latency);
criterion_main!(benches);

