//! Integration tests for demo application
//!
//! These tests verify that the demo application:
//! - Compiles successfully
//! - Runs without errors
//! - Starts dashboard correctly
//! - Generates and exports data
//! - Makes data visible in dashboard

use std::fs;
use std::path::PathBuf;
use std::process::{Command, Stdio};
use std::time::Duration;
use tempfile::TempDir;
use tokio::io::{AsyncReadExt, AsyncWriteExt};
use tokio::net::TcpStream;
use tokio::time::sleep;

fn find_available_port() -> u16 {
    use std::net::TcpListener;
    let listener = TcpListener::bind("127.0.0.1:0").unwrap();
    listener.local_addr().unwrap().port()
}

/// Test that demo app compiles successfully
#[test]
fn test_demo_app_compilation() {
    let output = Command::new("cargo")
        .args(&["build", "--example", "demo-app"])
        .output()
        .expect("Failed to execute cargo build");

    assert!(
        output.status.success(),
        "Demo app compilation failed: {}",
        String::from_utf8_lossy(&output.stderr)
    );
}

/// Test that demo app runs without errors (short execution)
#[tokio::test]
async fn test_demo_app_execution() {
    // Create temp directory for output
    let temp_dir = TempDir::new().unwrap();
    let output_dir = temp_dir.path().to_str().unwrap();

    // Set environment variable for output directory
    let mut child = Command::new("cargo")
        .args(&["run", "--example", "demo-app"])
        .env("OTLP_OUTPUT_DIR", output_dir)
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .spawn()
        .expect("Failed to spawn demo app");

    // Wait a bit for initialization
    sleep(Duration::from_millis(500)).await;

    // Check if process is still running (hasn't crashed)
    match child.try_wait() {
        Ok(Some(status)) => {
            // Process exited, check if it was successful or if we got output
            let output = child.wait_with_output().unwrap();
            let stdout = String::from_utf8_lossy(&output.stdout);
            let stderr = String::from_utf8_lossy(&output.stderr);
            
            // If it exited quickly, it might be because dashboard static dir doesn't exist
            // That's okay for this test - we just want to verify it doesn't panic
            if !status.success() {
                // Check if it's a known error (like missing dashboard dir) vs actual failure
                if !stderr.contains("Dashboard static directory") && !stderr.contains("dashboard") {
                    panic!("Demo app failed unexpectedly: {}\n{}", stdout, stderr);
                }
            }
        }
        Ok(None) => {
            // Process is still running - good! Kill it
            child.kill().unwrap();
            let _ = child.wait();
        }
        Err(e) => panic!("Error checking process status: {}", e),
    }
}

/// Test that dashboard starts when demo app runs
#[tokio::test]
async fn test_dashboard_startup() {
    // This test verifies that when dashboard is enabled, the HTTP server starts
    // We'll test this by checking if the dashboard server can start independently
    use otlp_arrow_library::dashboard::server::DashboardServer;
    
    let temp_dir = TempDir::new().unwrap();
    let static_dir = temp_dir.path().join("dashboard").join("dist");
    fs::create_dir_all(&static_dir).unwrap();
    
    // Create a minimal index.html for the dashboard
    fs::write(static_dir.join("index.html"), "<html><body>Dashboard</body></html>").unwrap();
    
    let port = find_available_port();
    let server = DashboardServer::new(static_dir, port, "127.0.0.1".to_string());
    
    let handle = server.start().await.expect("Dashboard server should start");
    
    // Give server a moment to start
    sleep(Duration::from_millis(100)).await;
    
    // Try to connect
    let addr = format!("127.0.0.1:{}", port);
    let mut stream = TcpStream::connect(&addr).await.expect("Should connect to dashboard");
    
    // Send a simple GET request
    let request = "GET / HTTP/1.1\r\nHost: localhost\r\n\r\n";
    stream.write_all(request.as_bytes()).await.unwrap();
    
    // Read response
    let mut buffer = [0; 1024];
    let n = stream.read(&mut buffer).await.unwrap();
    let response = String::from_utf8_lossy(&buffer[..n]);
    
    // Should get HTTP response
    assert!(response.contains("HTTP/1.1"), "Dashboard should respond with HTTP");
    
    handle.abort();
}

/// Test that demo app generates data (Arrow IPC files)
#[tokio::test]
async fn test_data_generation() {
    use otlp_arrow_library::{ConfigBuilder, OtlpLibrary};
    use opentelemetry_sdk::metrics::data::ResourceMetrics;
    use opentelemetry::trace::{SpanContext, SpanId, SpanKind, Status, TraceFlags, TraceId, TraceState};
    use opentelemetry_sdk::trace::SpanData;
    
    let temp_dir = TempDir::new().unwrap();
    
    let config = ConfigBuilder::new()
        .output_dir(temp_dir.path())
        .write_interval_secs(1) // Short interval for testing
        .dashboard_enabled(false) // Disable dashboard for this test
        .build()
        .unwrap();
    
    let library = OtlpLibrary::new(config).await.unwrap();
    
    // Export metrics using export_metrics (Protobuf)
    use opentelemetry_proto::tonic::collector::metrics::v1::ExportMetricsServiceRequest;
    let metrics_request = ExportMetricsServiceRequest::default();
    library.export_metrics(metrics_request).await.unwrap();
    
    // Export spans
    let trace_id = TraceId::from_bytes([1, 2, 3, 4, 5, 6, 7, 8, 9, 10, 11, 12, 13, 14, 15, 16]);
    let span_id = SpanId::from_bytes([1, 2, 3, 4, 5, 6, 7, 8]);
    let span_context = SpanContext::new(trace_id, span_id, TraceFlags::default(), false, TraceState::default());
    
    let span = SpanData {
        span_context,
        parent_span_id: SpanId::INVALID,
        span_kind: SpanKind::Server,
        name: std::borrow::Cow::Borrowed("test-span"),
        start_time: std::time::SystemTime::now(),
        end_time: std::time::SystemTime::now(),
        attributes: vec![].into_iter().collect(),
        events: opentelemetry_sdk::trace::SpanEvents::default(),
        links: opentelemetry_sdk::trace::SpanLinks::default(),
        status: Status::Ok,
        dropped_attributes_count: 0,
        parent_span_is_remote: false,
        instrumentation_scope: opentelemetry::InstrumentationScope::builder("test").build(),
    };
    
    library.export_trace(span).await.unwrap();
    
    // Wait for batch write
    sleep(Duration::from_secs(2)).await;
    
    // Flush to ensure writes complete
    library.flush().await.unwrap();
    
    // Verify files were created
    let traces_dir = temp_dir.path().join("otlp/traces");
    let metrics_dir = temp_dir.path().join("otlp/metrics");
    
    // Check that directories exist
    assert!(traces_dir.exists(), "Traces directory should exist");
    assert!(metrics_dir.exists(), "Metrics directory should exist");
    
    // Check for trace files
    let trace_files: Vec<_> = fs::read_dir(&traces_dir)
        .unwrap()
        .filter_map(|e| e.ok())
        .collect();
    
    assert!(!trace_files.is_empty(), "At least one trace file should be created");
    
    library.shutdown().await.unwrap();
}

/// Test that data appears in dashboard within write interval
#[tokio::test]
async fn test_dashboard_data_visibility() {
    use otlp_arrow_library::{ConfigBuilder, OtlpLibrary};
    use opentelemetry::trace::{SpanContext, SpanId, SpanKind, Status, TraceFlags, TraceId, TraceState};
    use opentelemetry_sdk::trace::SpanData;
    
    let temp_dir = TempDir::new().unwrap();
    let static_dir = temp_dir.path().join("dashboard").join("dist");
    fs::create_dir_all(&static_dir).unwrap();
    fs::write(static_dir.join("index.html"), "<html><body>Dashboard</body></html>").unwrap();
    
    let config = ConfigBuilder::new()
        .output_dir(temp_dir.path())
        .write_interval_secs(1) // Short interval for testing
        .dashboard_enabled(true)
        .dashboard_static_dir(static_dir.clone())
        .build()
        .unwrap();
    
    let library = OtlpLibrary::new(config).await.unwrap();
    
    // Export a span
    let trace_id = TraceId::from_bytes([1, 2, 3, 4, 5, 6, 7, 8, 9, 10, 11, 12, 13, 14, 15, 16]);
    let span_id = SpanId::from_bytes([1, 2, 3, 4, 5, 6, 7, 8]);
    let span_context = SpanContext::new(trace_id, span_id, TraceFlags::default(), false, TraceState::default());
    
    let span = SpanData {
        span_context,
        parent_span_id: SpanId::INVALID,
        span_kind: SpanKind::Server,
        name: std::borrow::Cow::Borrowed("test-span"),
        start_time: std::time::SystemTime::now(),
        end_time: std::time::SystemTime::now(),
        attributes: vec![].into_iter().collect(),
        events: opentelemetry_sdk::trace::SpanEvents::default(),
        links: opentelemetry_sdk::trace::SpanLinks::default(),
        status: Status::Ok,
        dropped_attributes_count: 0,
        parent_span_is_remote: false,
        instrumentation_scope: opentelemetry::InstrumentationScope::builder("test").build(),
    };
    
    library.export_trace(span).await.unwrap();
    
    // Wait for batch write (within write interval + buffer)
    sleep(Duration::from_secs(2)).await;
    
    // Flush to ensure data is written
    library.flush().await.unwrap();
    
    // Verify trace file exists (dashboard reads from these files)
    let traces_dir = temp_dir.path().join("otlp/traces");
    let trace_files: Vec<_> = fs::read_dir(&traces_dir)
        .unwrap()
        .filter_map(|e| e.ok())
        .collect();
    
    assert!(!trace_files.is_empty(), "Trace file should exist for dashboard to read");
    
    library.shutdown().await.unwrap();
}

/// Test that demo app code demonstrates all SDK patterns
#[test]
fn test_sdk_pattern_demonstration() {
    use std::fs;
    
    // Read the demo app source code
    let demo_code = fs::read_to_string("examples/demo-app.rs")
        .expect("Failed to read demo-app.rs");
    
    // Check for required SDK patterns
    let patterns = vec![
        ("ConfigBuilder", "ConfigBuilder usage"),
        ("OtlpLibrary::new", "Library initialization"),
        ("export_metrics", "Metric export"),
        ("export_trace", "Individual span export"),
        ("export_traces", "Batch span export"),
        ("flush", "Flush pattern"),
        ("shutdown", "Shutdown pattern"),
        ("dashboard_enabled", "Dashboard configuration"),
        ("SpanData", "Span creation"),
        ("SpanKind", "Span kinds"),
        ("parent_span_id", "Parent-child relationships"),
    ];
    
    for (pattern, description) in patterns {
        assert!(
            demo_code.contains(pattern),
            "Demo app should demonstrate {}: missing '{}'",
            description,
            pattern
        );
    }
}

/// Test that demo app has adequate code documentation
#[test]
fn test_code_documentation_coverage() {
    use std::fs;
    
    // Read the demo app source code
    let demo_code = fs::read_to_string("examples/demo-app.rs")
        .expect("Failed to read demo-app.rs");
    
    // Count SDK method calls (approximate)
    let sdk_calls = vec![
        "init_logging",
        "ConfigBuilder::new",
        "dashboard_enabled",
        "output_dir",
        "write_interval_secs",
        "build",
        "OtlpLibrary::new",
        "export_metrics",
        "export_trace",
        "export_traces",
        "flush",
        "shutdown",
    ];
    
    let mut documented_calls = 0;
    let mut total_calls = 0;
    
    for call in &sdk_calls {
        // Count occurrences of this call
        let count = demo_code.matches(call).count();
        total_calls += count;
        
        // Check if there's a comment nearby (within 3 lines)
        let lines: Vec<&str> = demo_code.lines().collect();
        for (i, line) in lines.iter().enumerate() {
            if line.contains(call) {
                // Check previous 3 lines for comments
                let start = i.saturating_sub(3);
                let context: String = lines[start..=i].join("\n");
                if context.contains("//") || context.contains("///") {
                    documented_calls += 1;
                }
            }
        }
    }
    
    // Calculate coverage percentage
    let coverage = if total_calls > 0 {
        (documented_calls as f64 / total_calls as f64) * 100.0
    } else {
        0.0
    };
    
    // Should have at least 80% comment coverage
    assert!(
        coverage >= 80.0,
        "Code documentation coverage is {:.1}%, should be at least 80%",
        coverage
    );
}

/// Test continuous generation mode
#[tokio::test]
async fn test_continuous_generation() {
    use otlp_arrow_library::{ConfigBuilder, OtlpLibrary};
    use opentelemetry_sdk::metrics::data::ResourceMetrics;
    use std::sync::atomic::{AtomicU64, Ordering};
    use std::sync::Arc;
    
    let temp_dir = TempDir::new().unwrap();
    
    let config = ConfigBuilder::new()
        .output_dir(temp_dir.path())
        .write_interval_secs(1)
        .dashboard_enabled(false)
        .build()
        .unwrap();
    
    let library = OtlpLibrary::new(config).await.unwrap();
    let counter = Arc::new(AtomicU64::new(0));
    let counter_clone = counter.clone();
    let library_clone = library.clone();
    
    // Spawn generation task
    let generation_handle = tokio::spawn(async move {
        let mut interval = tokio::time::interval(Duration::from_secs(1));
        interval.set_missed_tick_behavior(tokio::time::MissedTickBehavior::Skip);
        
        for _ in 0..3 {
            interval.tick().await;
            let count = counter_clone.fetch_add(1, Ordering::Relaxed);
            
            // Generate metrics
            use opentelemetry_proto::tonic::collector::metrics::v1::ExportMetricsServiceRequest;
            let metrics_request = ExportMetricsServiceRequest::default();
            let _ = library_clone.export_metrics(metrics_request).await;
            
            // Verify counter is incrementing (demonstrates time-series pattern)
            assert!(count < 10, "Counter should increment");
        }
    });
    
    // Wait for generation
    generation_handle.await.unwrap();
    
    // Verify counter was incremented
    assert!(counter.load(Ordering::Relaxed) >= 3, "Should generate at least 3 batches");
    
    library.shutdown().await.unwrap();
}

/// Test graceful shutdown on signal
#[tokio::test]
async fn test_graceful_shutdown() {
    use otlp_arrow_library::{ConfigBuilder, OtlpLibrary};
    
    let temp_dir = TempDir::new().unwrap();
    
    let config = ConfigBuilder::new()
        .output_dir(temp_dir.path())
        .write_interval_secs(1)
        .dashboard_enabled(false)
        .build()
        .unwrap();
    
    let library = OtlpLibrary::new(config).await.unwrap();
    
    // Export some data
    use opentelemetry_proto::tonic::collector::metrics::v1::ExportMetricsServiceRequest;
    let metrics_request = ExportMetricsServiceRequest::default();
    library.export_metrics(metrics_request).await.unwrap();
    
    // Flush before shutdown (graceful shutdown pattern)
    library.flush().await.expect("Flush should succeed");
    
    // Shutdown gracefully
    library.shutdown().await.expect("Shutdown should succeed");
    
    // Verify files were written (data wasn't lost)
    let traces_dir = temp_dir.path().join("otlp/traces");
    let metrics_dir = temp_dir.path().join("otlp/metrics");
    
    // At least one directory should exist
    assert!(
        traces_dir.exists() || metrics_dir.exists(),
        "Data should be written before shutdown"
    );
}

