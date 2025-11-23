//! Performance benchmark for format conversion overhead

use criterion::{black_box, criterion_group, criterion_main, Criterion};
use otlp_arrow_library::otlp::converter::FormatConverter;
use opentelemetry_proto::tonic::collector::trace::v1::ExportTraceServiceRequest;
use opentelemetry_sdk::trace::SpanData;
use opentelemetry::trace::{SpanContext, SpanId, SpanKind, Status, TraceId, TraceFlags, TraceState};
use std::time::Instant;

fn create_test_span(i: usize) -> SpanData {
    let trace_id = TraceId::from_bytes([i as u8; 16]);
    let span_id = SpanId::from_bytes([i as u8; 8]);
    let span_context = SpanContext::new(trace_id, span_id, TraceFlags::default(), false, TraceState::default());

    SpanData {
        span_context,
        parent_span_id: SpanId::INVALID,
        span_kind: SpanKind::Internal,
        name: std::borrow::Cow::Owned(format!("conversion-span-{}", i)),
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

fn bench_format_conversion(c: &mut Criterion) {
    let converter = FormatConverter::new();
    
    c.bench_function("protobuf_to_arrow_flight_traces", |b| {
        let request = ExportTraceServiceRequest::default();
        
        b.iter(|| {
            let start = Instant::now();
            let _ = converter.protobuf_to_arrow_flight_traces(black_box(&request));
            let elapsed = start.elapsed();
            black_box(elapsed);
        });
    });

    c.bench_function("spans_to_arrow_batch", |b| {
        let spans: Vec<SpanData> = (0..100).map(|i| create_test_span(i)).collect();
        
        b.iter(|| {
            let start = Instant::now();
            let _ = FormatConverter::spans_to_arrow_batch(black_box(&spans));
            let elapsed = start.elapsed();
            black_box(elapsed);
        });
    });
}

criterion_group!(benches, bench_format_conversion);
criterion_main!(benches);

