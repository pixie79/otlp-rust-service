//! Embedded library usage example
//!
//! This example demonstrates how to use the OTLP library as an embedded component
//! in another Rust application, using the public API to export traces and metrics.

use opentelemetry::trace::{
    SpanContext, SpanId, SpanKind, Status, TraceFlags, TraceId, TraceState,
};
use opentelemetry_sdk::trace::SpanData;
use otlp_arrow_library::{Config, ConfigBuilder, OtlpLibrary};
use std::path::PathBuf;
use tempfile::TempDir;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Initialize logging
    otlp_arrow_library::init_logging();

    // Create a temporary directory for output (in production, use a real path)
    let temp_dir = TempDir::new()?;

    // Build configuration using ConfigBuilder
    let config = ConfigBuilder::new()
        .output_dir(temp_dir.path())
        .write_interval_secs(2)
        .trace_cleanup_interval_secs(600)
        .metric_cleanup_interval_secs(3600)
        .protobuf_enabled(true)
        .arrow_flight_enabled(true)
        .build()?;

    // Create library instance
    let library = OtlpLibrary::new(config).await?;

    // Export a single trace span
    let trace_id = TraceId::from_bytes([1, 2, 3, 4, 5, 6, 7, 8, 9, 10, 11, 12, 13, 14, 15, 16]);
    let span_id = SpanId::from_bytes([1, 2, 3, 4, 5, 6, 7, 8]);
    let span_context = SpanContext::new(
        trace_id,
        span_id,
        TraceFlags::default(),
        false,
        TraceState::default(),
    );

    let span = SpanData {
        span_context,
        parent_span_id: SpanId::INVALID,
        span_kind: SpanKind::Server,
        name: std::borrow::Cow::Borrowed("example-operation"),
        start_time: std::time::SystemTime::now(),
        end_time: std::time::SystemTime::now(),
        attributes: vec![
            opentelemetry::KeyValue::new("service.name", "example-service"),
            opentelemetry::KeyValue::new("http.method", "GET"),
            opentelemetry::KeyValue::new("http.status_code", 200),
        ]
        .into_iter()
        .collect(),
        events: opentelemetry_sdk::trace::SpanEvents::default(),
        links: opentelemetry_sdk::trace::SpanLinks::default(),
        status: Status::Ok,
        dropped_attributes_count: 0,
        parent_span_is_remote: false,
        instrumentation_scope: opentelemetry::InstrumentationScope::builder("example")
            .with_version("1.0.0")
            .build(),
    };

    library.export_trace(span).await?;

    // Export multiple spans
    let mut spans = Vec::new();
    for i in 0..5 {
        let trace_id = TraceId::from_bytes([i as u8; 16]);
        let span_id = SpanId::from_bytes([i as u8; 8]);
        let span_context = SpanContext::new(
            trace_id,
            span_id,
            TraceFlags::default(),
            false,
            TraceState::default(),
        );

        let span = SpanData {
            span_context,
            parent_span_id: SpanId::INVALID,
            span_kind: SpanKind::Internal,
            name: std::borrow::Cow::Owned(format!("batch-operation-{}", i)),
            start_time: std::time::SystemTime::now(),
            end_time: std::time::SystemTime::now(),
            attributes: vec![].into_iter().collect(),
            events: opentelemetry_sdk::trace::SpanEvents::default(),
            links: opentelemetry_sdk::trace::SpanLinks::default(),
            status: Status::Ok,
            dropped_attributes_count: 0,
            parent_span_is_remote: false,
            instrumentation_scope: opentelemetry::InstrumentationScope::builder("example").build(),
        };
        spans.push(span);
    }

    library.export_traces(spans).await?;

    // Export metrics
    let metrics = opentelemetry_sdk::metrics::data::ResourceMetrics::default();
    library.export_metrics(metrics).await?;

    // Wait a bit for batch writing
    tokio::time::sleep(tokio::time::Duration::from_secs(3)).await;

    // Force flush to ensure all data is written
    library.flush().await?;

    println!(
        "Exported traces and metrics. Check output directory: {}",
        temp_dir.path().display()
    );

    // Shutdown gracefully
    library.shutdown().await?;

    Ok(())
}
