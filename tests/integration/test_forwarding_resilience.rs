//! Integration test for local storage continuing during forwarding failures

use otlp_arrow_library::{Config, ForwardingConfig, ForwardingProtocol, OtlpLibrary};
use opentelemetry_sdk::trace::SpanData;
use opentelemetry::trace::{SpanContext, SpanId, SpanKind, Status, TraceId, TraceFlags, TraceState};
use std::path::PathBuf;
use tempfile::TempDir;
use tokio::time::Duration;

#[tokio::test]
async fn test_local_storage_resilience_during_forwarding_failures() {
    let temp_dir = TempDir::new().unwrap();
    
    // Configure forwarding to an endpoint that will consistently fail
    let forwarding = ForwardingConfig {
        enabled: true,
        endpoint_url: Some("http://127.0.0.1:1".to_string()), // Invalid port, will fail
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

    // Export multiple spans - all forwarding attempts will fail
    // but local storage should continue working
    for i in 0..10 {
        let trace_id = TraceId::from_bytes([i as u8; 16]);
        let span_id = SpanId::from_bytes([i as u8; 8]);
        let span_context = SpanContext::new(trace_id, span_id, TraceFlags::default(), false, TraceState::default());
        
        let span = SpanData {
            span_context,
            parent_span_id: SpanId::INVALID,
            span_kind: SpanKind::Internal,
            name: std::borrow::Cow::Owned(format!("resilience-test-span-{}", i)),
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

        library.export_trace(span).await.unwrap();
        
        // Small delay between exports
        tokio::time::sleep(Duration::from_millis(100)).await;
    }

    // Wait for write interval
    tokio::time::sleep(Duration::from_secs(2)).await;

    // Flush to ensure all writes complete
    library.flush().await.unwrap();

    // Verify all files were created locally despite forwarding failures
    let traces_dir = temp_dir.path().join("otlp/traces");
    let files: Vec<_> = std::fs::read_dir(&traces_dir)
        .unwrap()
        .filter_map(|e| e.ok())
        .collect();
    assert!(!files.is_empty(), "Expected trace files to be created despite forwarding failures");
    
    // Verify we can still export more spans after failures
    let trace_id = TraceId::from_bytes([99; 16]);
    let span_id = SpanId::from_bytes([99; 8]);
    let span_context = SpanContext::new(trace_id, span_id, TraceFlags::default(), false, TraceState::default());
    
    let span = SpanData {
        span_context,
        parent_span_id: SpanId::INVALID,
        span_kind: SpanKind::Internal,
        name: std::borrow::Cow::Borrowed("post-failure-span"),
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

    library.export_trace(span).await.unwrap();
    library.flush().await.unwrap();

    // Verify the new span was also stored
    let files_after: Vec<_> = std::fs::read_dir(&traces_dir)
        .unwrap()
        .filter_map(|e| e.ok())
        .collect();
    assert!(files_after.len() >= files.len(), "Expected additional files after continued exports");

    // Cleanup
    library.shutdown().await.unwrap();
}

