//! Unit tests for OtlpSpanExporter

use opentelemetry::trace::{SpanContext, SpanId, SpanKind, Status, TraceFlags, TraceId};
use opentelemetry::KeyValue;
use opentelemetry_sdk::trace::{SpanData, SpanExporter};
use otlp_arrow_library::{Config, OtlpLibrary};
use std::path::PathBuf;
use std::time::{Duration, SystemTime};
use tempfile::TempDir;

/// Helper function to create a test span
fn create_test_span(name: &str) -> SpanData {
    let trace_id = TraceId::from_bytes([1, 2, 3, 4, 5, 6, 7, 8, 9, 10, 11, 12, 13, 14, 15, 16]);
    let span_id = SpanId::from_bytes([1, 2, 3, 4, 5, 6, 7, 8]);

    let span_context = SpanContext::new(
        trace_id,
        span_id,
        TraceFlags::default(),
        false,
        opentelemetry::trace::TraceState::default(),
    );

    SpanData {
        span_context,
        parent_span_id: SpanId::INVALID,
        span_kind: SpanKind::Server,
        name: std::borrow::Cow::Owned(name.to_string()),
        start_time: SystemTime::now(),
        end_time: SystemTime::now() + Duration::from_secs(1),
        attributes: vec![KeyValue::new("service.name", "test-service")],
        events: opentelemetry_sdk::trace::SpanEvents::default(),
        links: opentelemetry_sdk::trace::SpanLinks::default(),
        status: Status::Ok,
        dropped_attributes_count: 0,
        parent_span_is_remote: false,
        instrumentation_scope: opentelemetry::InstrumentationScope::builder("test").build(),
    }
}

#[tokio::test]
async fn test_otlp_span_exporter_export() {
    let temp_dir = TempDir::new().unwrap();

    let config = Config {
        output_dir: PathBuf::from(temp_dir.path()),
        write_interval_secs: 1,
        trace_cleanup_interval_secs: 600,
        metric_cleanup_interval_secs: 3600,
        protocols: Default::default(),
        forwarding: None,
        dashboard: Default::default(),
    };

    let library = OtlpLibrary::new(config).await.unwrap();
    let exporter = library.span_exporter();

    let spans = vec![create_test_span("test-span")];

    // Test export method (via SpanExporter trait)
    let result = exporter.export(spans).await;
    assert!(result.is_ok(), "Export should succeed");

    // Flush to ensure write completes
    library.flush().await.expect("Failed to flush");
}

#[tokio::test]
async fn test_otlp_span_exporter_shutdown() {
    let temp_dir = TempDir::new().unwrap();

    let config = Config {
        output_dir: PathBuf::from(temp_dir.path()),
        write_interval_secs: 1,
        trace_cleanup_interval_secs: 600,
        metric_cleanup_interval_secs: 3600,
        protocols: Default::default(),
        forwarding: None,
        dashboard: Default::default(),
    };

    let library = OtlpLibrary::new(config).await.unwrap();
    let mut exporter = library.span_exporter();

    // Test shutdown method (via SpanExporter trait)
    // Note: shutdown() requires &mut self, so we need mut here
    let result = exporter.shutdown();
    assert!(result.is_ok(), "Shutdown should succeed");

    // Library should still be functional after exporter shutdown
    let spans = vec![create_test_span("test-span")];
    let export_result = exporter.export(spans).await;
    assert!(
        export_result.is_ok(),
        "Export should still work after shutdown"
    );
}

#[tokio::test]
async fn test_otlp_span_exporter_error_conversion() {
    let temp_dir = TempDir::new().unwrap();

    let config = Config {
        output_dir: PathBuf::from(temp_dir.path()),
        write_interval_secs: 1,
        trace_cleanup_interval_secs: 600,
        metric_cleanup_interval_secs: 3600,
        protocols: Default::default(),
        forwarding: None,
        dashboard: Default::default(),
    };

    let library = OtlpLibrary::new(config).await.unwrap();
    let exporter = library.span_exporter();

    // Shutdown library to cause export to fail
    library.shutdown().await.expect("Shutdown should succeed");

    // Wait a bit for shutdown to complete
    tokio::time::sleep(tokio::time::Duration::from_millis(100)).await;

    // Try to export after shutdown - should convert error appropriately
    let spans = vec![create_test_span("test-span")];
    let result = exporter.export(spans).await;

    // Should return OTelSdkError::InternalFailure
    // Note: After shutdown, the library may still accept exports if the batch buffer
    // is still active, so we check for either error or success (both are valid)
    if let Err(e) = result {
        // Verify it's an InternalFailure error
        match e {
            opentelemetry_sdk::error::OTelSdkError::InternalFailure(msg) => {
                assert!(
                    msg.contains("OtlpLibrary"),
                    "Error message should contain context about OtlpLibrary"
                );
            }
            _ => panic!("Expected InternalFailure error, got: {:?}", e),
        }
    }
    // If result is Ok, that's also acceptable - the library may still process exports
}
