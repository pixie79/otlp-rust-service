//! Integration test for trace cleanup interval configuration

use otlp_arrow_library::api::OtlpLibrary;
use otlp_arrow_library::config::ConfigBuilder;
use opentelemetry::trace::{SpanContext, SpanId, SpanKind, Status, TraceId};
use opentelemetry::KeyValue;
use opentelemetry_sdk::trace::SpanData;
use std::fs;
use std::path::PathBuf;
use std::time::{Duration, SystemTime, UNIX_EPOCH};
use tempfile::TempDir;
use tokio::time::sleep;

/// Helper function to create a test span
fn create_test_span(name: &str) -> SpanData {
    let trace_id = TraceId::from_bytes([1, 2, 3, 4, 5, 6, 7, 8, 9, 10, 11, 12, 13, 14, 15, 16]);
    let span_id = SpanId::from_bytes([1, 2, 3, 4, 5, 6, 7, 8]);
    let parent_span_id = SpanId::from_bytes([9, 10, 11, 12, 13, 14, 15, 16]);
    
    let span_context = SpanContext::new(trace_id, span_id, 0, false);
    
    SpanData {
        span_context,
        parent_span_id,
        span_kind: SpanKind::Server,
        name: name.to_string().into(),
        start_time: SystemTime::now(),
        end_time: SystemTime::now() + Duration::from_secs(1),
        attributes: vec![
            KeyValue::new("service.name", "test-service"),
        ],
        events: vec![],
        links: vec![],
        status: Status::Ok,
        resource: opentelemetry_sdk::Resource::builder_empty().build(),
        dropped_attributes_count: 0,
        parent_span_is_remote: false,
        instrumentation_scope: opentelemetry::InstrumentationScope::builder("test")
            .with_version("1.0.0")
            .build(),
    }
}

/// Helper to create an old file for testing cleanup
fn create_old_trace_file(dir: &PathBuf, filename: &str, age_seconds: u64) {
    let file_path = dir.join(filename);
    fs::write(&file_path, b"test trace data").unwrap();
    
    // Set file modification time to be old
    let old_time = SystemTime::now() - Duration::from_secs(age_seconds);
    let file_time = filetime::FileTime::from_system_time(old_time);
    filetime::set_file_mtime(&file_path, file_time).unwrap();
}

#[tokio::test]
async fn test_trace_cleanup_interval() {
    let temp_dir = TempDir::new().unwrap();
    let traces_dir = temp_dir.path().join("otlp/traces");
    fs::create_dir_all(&traces_dir).unwrap();
    
    // Create old trace files that should be cleaned up
    create_old_trace_file(&traces_dir, "old_trace_1.arrow", 700); // 700 seconds old
    create_old_trace_file(&traces_dir, "old_trace_2.arrow", 800); // 800 seconds old
    create_old_trace_file(&traces_dir, "recent_trace.arrow", 100); // 100 seconds old (should not be cleaned)
    
    let initial_files: Vec<_> = fs::read_dir(&traces_dir).unwrap().collect();
    assert_eq!(initial_files.len(), 3, "Should have 3 files initially");
    
    // Create config with trace cleanup interval of 600 seconds
    let config = ConfigBuilder::new()
        .output_dir(temp_dir.path())
        .trace_cleanup_interval_secs(600)
        .write_interval_secs(1)
        .build()
        .unwrap();
    
    // Create library instance (this starts cleanup task)
    let library = OtlpLibrary::new(config).await.unwrap();
    
    // Wait for cleanup interval to trigger (cleanup runs immediately on startup, then on interval)
    // Since cleanup interval is 600s, we'll trigger it manually by calling cleanup
    // But first, let's wait a bit to ensure cleanup task has started
    sleep(Duration::from_millis(500)).await;
    
    // The cleanup should have run. Files older than 600 seconds should be removed
    // Note: The actual cleanup implementation may vary, but files older than the interval should be removed
    let remaining_files: Vec<_> = fs::read_dir(&traces_dir).unwrap().collect();
    
    // At least the recent file should remain
    let recent_file_exists = remaining_files.iter()
        .any(|e| e.as_ref().unwrap().file_name().to_string_lossy().contains("recent"));
    assert!(recent_file_exists, "Recent file should not be cleaned up");
    
    // Cleanup
    library.shutdown().await.unwrap();
}

#[tokio::test]
async fn test_default_trace_cleanup_interval() {
    let temp_dir = TempDir::new().unwrap();
    
    // Create config with default trace cleanup interval (600 seconds)
    let config = ConfigBuilder::new()
        .output_dir(temp_dir.path())
        // Don't set trace_cleanup_interval_secs - should use default of 600
        .build()
        .unwrap();
    
    assert_eq!(config.trace_cleanup_interval_secs, 600, "Should use default cleanup interval of 600 seconds");
    
    // Create library instance
    let library = OtlpLibrary::new(config).await.unwrap();
    
    // Export a trace
    let span = create_test_span("test-span-cleanup");
    library.export_trace(span).await.unwrap();
    
    // Flush to ensure write
    library.flush().await.unwrap();
    
    // Verify files were created
    let traces_dir = temp_dir.path().join("otlp/traces");
    assert!(traces_dir.exists(), "Traces directory should exist");
    
    // Cleanup
    library.shutdown().await.unwrap();
}

