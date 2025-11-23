//! Integration test for forwarding enabled with Protobuf endpoint

use otlp_arrow_library::{Config, ForwardingConfig, ForwardingProtocol, OtlpLibrary};
use opentelemetry_sdk::trace::SpanData;
use opentelemetry::trace::{SpanContext, SpanId, SpanKind, Status, TraceId, TraceFlags, TraceState};
use std::path::PathBuf;
use tempfile::TempDir;
use tokio::time::Duration;

#[tokio::test]
async fn test_forwarding_enabled_protobuf() {
    let temp_dir = TempDir::new().unwrap();
    
    // Configure forwarding to a non-existent endpoint (will fail, but tests the code path)
    let forwarding = ForwardingConfig {
        enabled: true,
        endpoint_url: Some("http://localhost:9999".to_string()), // Non-existent endpoint
        protocol: ForwardingProtocol::Protobuf,
        authentication: None,
    };

    let config = Config {
        output_dir: PathBuf::from(temp_dir.path()),
        write_interval_secs: 1,
        trace_cleanup_interval_secs: 600,
        metric_cleanup_interval_secs: 3600,
        protocols: Default::default(),
        forwarding: Some(forwarding),
    };

    let library = OtlpLibrary::new(config).await.unwrap();

    // Create a test span
    let trace_id = TraceId::from_bytes([1; 16]);
    let span_id = SpanId::from_bytes([1; 8]);
    let span_context = SpanContext::new(trace_id, span_id, TraceFlags::default(), false, TraceState::default());
    
    let span = SpanData {
        span_context,
        parent_span_id: SpanId::INVALID,
        span_kind: SpanKind::Internal,
        name: std::borrow::Cow::Borrowed("test-span"),
        start_time: std::time::SystemTime::now(),
        end_time: std::time::SystemTime::now(),
        attributes: vec![].into_iter().collect(),
        events: opentelemetry_sdk::trace::SpanEvents::default(),
        links: opentelemetry_sdk::trace::SpanLinks::default(),
        status: Status::Ok,
        dropped_attributes_count: 0,
        parent_span_is_remote: false,
        instrumentation_scope: opentelemetry::InstrumentationScope::builder("test")
            .build(),
    };

    // Export span - forwarding will fail but local storage should succeed
    library.export_trace(span).await.unwrap();

    // Wait for write interval
    tokio::time::sleep(Duration::from_secs(2)).await;

    // Flush to ensure write
    library.flush().await.unwrap();

    // Verify file was created locally (forwarding failure shouldn't prevent local storage)
    let traces_dir = temp_dir.path().join("otlp/traces");
    let files: Vec<_> = std::fs::read_dir(&traces_dir)
        .unwrap()
        .filter_map(|e| e.ok())
        .collect();
    assert!(!files.is_empty(), "Expected trace file to be created even if forwarding fails");

    // Cleanup
    library.shutdown().await.unwrap();
}

