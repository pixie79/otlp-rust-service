//! Integration test for forwarding failure handling

use otlp_arrow_library::{Config, ForwardingConfig, ForwardingProtocol, OtlpLibrary};
use opentelemetry_sdk::trace::SpanData;
use opentelemetry::trace::{SpanContext, SpanId, SpanKind, Status, TraceId, TraceFlags, TraceState};
use std::path::PathBuf;
use tempfile::TempDir;
use tokio::time::Duration;

#[tokio::test]
async fn test_forwarding_failure_does_not_block_local_storage() {
    let temp_dir = TempDir::new().unwrap();
    
    // Configure forwarding to an invalid endpoint that will fail
    let forwarding = ForwardingConfig {
        enabled: true,
        endpoint_url: Some("http://invalid-host-that-does-not-exist:9999".to_string()),
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

    // Create multiple test spans
    for i in 0..5 {
        let trace_id = TraceId::from_bytes([i as u8; 16]);
        let span_id = SpanId::from_bytes([i as u8; 8]);
        let span_context = SpanContext::new(trace_id, span_id, TraceFlags::default(), false, TraceState::default());
        
        let span = SpanData {
            span_context,
            parent_span_id: SpanId::INVALID,
            span_kind: SpanKind::Internal,
            name: std::borrow::Cow::Owned(format!("test-span-{}", i)),
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
    }

    // Wait for write interval
    tokio::time::sleep(Duration::from_secs(2)).await;

    // Flush to ensure write
    library.flush().await.unwrap();

    // Verify files were created locally despite forwarding failures
    let traces_dir = temp_dir.path().join("otlp/traces");
    let files: Vec<_> = std::fs::read_dir(&traces_dir)
        .unwrap()
        .filter_map(|e| e.ok())
        .collect();
    assert!(!files.is_empty(), "Expected trace files to be created even with forwarding failures");

    // Cleanup
    library.shutdown().await.unwrap();
}

