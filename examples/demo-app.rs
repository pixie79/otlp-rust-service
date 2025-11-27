//! Demo Rust Application for OTLP SDK
//!
//! This demo application demonstrates how to use the OTLP Arrow Library SDK to:
//! - Enable the dashboard for real-time telemetry visualization
//! - Create and export metrics directly without SDK (Path 2: Direct API - Protobuf)
//! - Create and export mock trace spans with relationships
//! - Serve as a reference implementation for developers
//!
//! The demo uses Import Path 2 from the flow diagram:
//! - Direct creation of InternalResourceMetrics → Protobuf → export_metrics() → Arrow RecordBatch → Arrow IPC file
//! - No SDK needed! No temporary server!
//!
//! ## Usage
//!
//! Run the demo application:
//! ```bash
//! cargo run --example demo-app
//! ```
//!
//! Then open your browser to http://127.0.0.1:8080 to view the dashboard.
//!
//! Press Ctrl+C to stop the demo gracefully.

use opentelemetry::KeyValue;
use opentelemetry::trace::{
    SpanContext, SpanId, SpanKind, Status, TraceFlags, TraceId, TraceState,
};
use opentelemetry_sdk::trace::SpanData;
use otlp_arrow_library::otlp::metrics_data::*;
use otlp_arrow_library::{ConfigBuilder, OtlpLibrary};
use std::sync::atomic::{AtomicU64, Ordering};
use std::time::{SystemTime, UNIX_EPOCH};
use tokio::signal;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    // ============================================================================
    // Initialization Section
    // ============================================================================

    // Initialize structured logging using the library's logging infrastructure
    // This sets up tracing with appropriate log levels and formatting
    otlp_arrow_library::init_logging();

    // Check if dashboard static directory exists before enabling dashboard
    // This provides a helpful error message if the dashboard hasn't been built
    let dashboard_static_dir = std::path::Path::new("./dashboard/dist");
    if !dashboard_static_dir.exists() {
        eprintln!("⚠️  Warning: Dashboard static directory not found at ./dashboard/dist");
        eprintln!("   The demo will run without the dashboard.");
        eprintln!("   To enable the dashboard, build it first:");
        eprintln!("   cd dashboard && npm install && npm run build && cd ..");
        eprintln!();
    }

    // Build configuration using ConfigBuilder pattern
    // This demonstrates how to configure the library with dashboard enabled
    // Note: If dashboard directory doesn't exist, dashboard will be automatically disabled
    // by the validation, but we try to enable it to demonstrate the pattern
    let mut config_builder = ConfigBuilder::new()
        // Set output directory for Arrow IPC files
        .output_dir("./output_dir")
        // Configure write interval (how often batches are written to disk)
        .write_interval_secs(5)
        // Set cleanup intervals for old files
        .trace_cleanup_interval_secs(600)
        .metric_cleanup_interval_secs(3600)
        // Enable both gRPC protocols (optional, for completeness)
        .protobuf_enabled(true)
        .arrow_flight_enabled(true);

    // Only enable dashboard if the static directory exists
    // This demonstrates conditional dashboard configuration
    if dashboard_static_dir.exists() {
        config_builder = config_builder
            // Enable dashboard HTTP server (default port 8080)
            .dashboard_enabled(true);
    } else {
        // Dashboard disabled - demo will still generate data
        config_builder = config_builder.dashboard_enabled(false);
    }

    let config = config_builder.build()?;

    // Store dashboard enabled status before moving config
    let dashboard_enabled = config.dashboard.enabled;

    // Create library instance with the configuration
    // This initializes the library, creates output directories, and starts
    // background tasks for batch writing and file cleanup
    // If dashboard is enabled, the HTTP server starts automatically
    let library = OtlpLibrary::new(config.clone()).await?;

    // ============================================================================
    // Direct Metrics Creation (No SDK Required!)
    // ============================================================================

    // Path 2: Direct API - Protobuf
    // Flow: InternalResourceMetrics → Protobuf → export_metrics() → Arrow RecordBatch → Arrow IPC file
    // No SDK! No temporary server!
    // We create metrics directly using our internal structures, convert to Protobuf, then export

    // Print dashboard URL for user convenience (if dashboard is enabled)
    println!("🚀 Demo application started!");
    if dashboard_enabled {
        println!("📊 Dashboard available at http://127.0.0.1:8080");
    } else {
        println!("📊 Dashboard disabled (static files not found)");
        println!("   Data will still be written to ./output_dir/otlp/");
    }
    println!("📝 Generating mock telemetry data...");
    println!("💡 Press Ctrl+C to stop gracefully\n");

    // ============================================================================
    // Continuous Generation Mode
    // ============================================================================

    // Use tokio::select! to handle both continuous data generation and shutdown signal
    // This allows the demo to run continuously until interrupted, generating data
    // at regular intervals to demonstrate time-series patterns in the dashboard
    let generation_counter = std::sync::Arc::new(AtomicU64::new(0));
    let counter_clone = generation_counter.clone();
    let library_clone = library.clone();

    // Spawn continuous generation task
    let generation_handle = tokio::spawn(async move {
        let mut interval = tokio::time::interval(tokio::time::Duration::from_secs(3));
        interval.set_missed_tick_behavior(tokio::time::MissedTickBehavior::Skip);

        loop {
            interval.tick().await;

            let count = counter_clone.fetch_add(1, Ordering::Relaxed);

            // ============================================================================
            // Create Metrics Directly (No SDK Required!)
            // ============================================================================

            // Get current timestamp in nanoseconds
            let now = SystemTime::now()
                .duration_since(UNIX_EPOCH)
                .unwrap()
                .as_nanos() as u64;

            // Create metrics directly using InternalResourceMetrics
            // This demonstrates Path 2: Direct API - Protobuf
            let metrics = InternalResourceMetrics {
                resource: InternalResource {
                    attributes: vec![
                        KeyValue::new("service.name", "demo-app"),
                        KeyValue::new("service.version", "1.0.0"),
                    ],
                    dropped_attributes_count: 0,
                },
                scope_metrics: vec![InternalScopeMetrics {
                    scope: InternalInstrumentationScope {
                        name: "demo-app".to_string(),
                        version: Some("1.0.0".to_string()),
                        attributes: vec![],
                        dropped_attributes_count: 0,
                    },
                    metrics: vec![
                        // Counter metric: http_requests_total
                        InternalMetric {
                            name: "http_requests_total".to_string(),
                            description: Some("Total number of HTTP requests".to_string()),
                            unit: Some("1".to_string()),
                            data: InternalMetricData::Sum(InternalSum {
                                data_points: vec![InternalNumberDataPoint {
                                    attributes: vec![
                                        KeyValue::new("method", "GET"),
                                        KeyValue::new("status", "200"),
                                        KeyValue::new("endpoint", "/api/users"),
                                    ],
                                    start_time_unix_nano: None,
                                    time_unix_nano: now,
                                    value: InternalNumberValue::AsInt((count + 1) as i64),
                                }],
                                aggregation_temporality: 2, // CUMULATIVE
                                is_monotonic: true,
                            }),
                        },
                        // Histogram metric: http_request_duration_seconds
                        InternalMetric {
                            name: "http_request_duration_seconds".to_string(),
                            description: Some("HTTP request duration in seconds".to_string()),
                            unit: Some("s".to_string()),
                            data: InternalMetricData::Histogram(InternalHistogram {
                                data_points: vec![InternalHistogramDataPoint {
                                    attributes: vec![
                                        KeyValue::new("method", "GET"),
                                        KeyValue::new("endpoint", "/api/users"),
                                    ],
                                    start_time_unix_nano: None,
                                    time_unix_nano: now,
                                    count: 1,
                                    sum: Some(0.05 + (count % 10) as f64 * 0.01),
                                    bucket_counts: vec![0, 1, 0, 0, 0],
                                    explicit_bounds: vec![0.01, 0.05, 0.1, 0.5],
                                    min: Some(0.05),
                                    max: Some(0.14),
                                }],
                                aggregation_temporality: 2, // CUMULATIVE
                            }),
                        },
                        // UpDownCounter metric: active_connections
                        InternalMetric {
                            name: "active_connections".to_string(),
                            description: Some("Number of active connections".to_string()),
                            unit: Some("1".to_string()),
                            data: InternalMetricData::Sum(InternalSum {
                                data_points: vec![InternalNumberDataPoint {
                                    attributes: vec![KeyValue::new("service", "demo-service")],
                                    start_time_unix_nano: None,
                                    time_unix_nano: now,
                                    value: InternalNumberValue::AsInt(10 + (count % 20) as i64),
                                }],
                                aggregation_temporality: 2, // CUMULATIVE
                                is_monotonic: false,
                            }),
                        },
                        // Gauge metric: cpu_usage_percent
                        InternalMetric {
                            name: "cpu_usage_percent".to_string(),
                            description: Some("CPU usage percentage".to_string()),
                            unit: Some("%".to_string()),
                            data: InternalMetricData::Gauge(InternalGauge {
                                data_points: vec![InternalNumberDataPoint {
                                    attributes: vec![KeyValue::new("host", "demo-host-1")],
                                    start_time_unix_nano: None,
                                    time_unix_nano: now,
                                    value: InternalNumberValue::AsDouble(
                                        20.0 + (count % 60) as f64,
                                    ),
                                }],
                            }),
                        },
                    ],
                    schema_url: String::new(),
                }],
                schema_url: String::new(),
            };

            // Convert to Protobuf and export (Path 2: Direct API - Protobuf)
            match metrics.to_protobuf() {
                Ok(protobuf_request) => {
                    if let Err(e) = library_clone.export_metrics(protobuf_request).await {
                        eprintln!("Error exporting metrics: {}", e);
                    }
                }
                Err(e) => {
                    eprintln!("Error converting metrics to Protobuf: {}", e);
                }
            }

            // Generate and export spans with varying attributes
            // This demonstrates trace visualization in the dashboard
            let spans = generate_mock_spans_with_counter(count);
            let span_count = spans.len();
            if let Err(e) = library_clone.export_traces(spans).await {
                eprintln!("Error exporting spans: {}", e);
            }

            println!(
                "✓ Generated batch #{} (direct metrics + {} spans)",
                count + 1,
                span_count
            );
        }
    });

    // Wait for Ctrl+C signal for graceful shutdown
    // This demonstrates proper signal handling and graceful shutdown pattern
    signal::ctrl_c().await?;
    println!("\n🛑 Shutdown signal received, shutting down gracefully...");

    // Cancel the generation task
    generation_handle.abort();

    // ============================================================================
    // Graceful Shutdown Section
    // ============================================================================

    // No MeterProvider to shutdown - we're using direct API!

    // Flush all pending data to ensure nothing is lost
    // This forces immediate write of any buffered data before shutdown
    library.flush().await?;
    println!("✓ Flushed pending data");

    // Shutdown gracefully
    // This stops background tasks, cleans up resources, and ensures
    // all data is written before the application exits
    library.shutdown().await?;
    println!("✓ Shutdown complete");

    println!("\n✅ Demo completed successfully!");
    println!("📁 Check ./output_dir/otlp/ for Arrow IPC files");
    if dashboard_enabled {
        println!("🌐 Dashboard available at http://127.0.0.1:8080");
        println!(
            "   📂 Select directory: ./output_dir/otlp (contains traces/ and metrics/ subdirectories)"
        );
        println!("   ✅ Direct metrics (no SDK) and traces should be visible in the dashboard");
    } else {
        println!("💡 To view data in dashboard, build it first:");
        println!("   cd dashboard && npm install && npm run build && cd ..");
        println!("   Then run the demo again.");
    }

    Ok(())
}

// Note: Metrics are created directly using InternalResourceMetrics (no SDK required!)
// This demonstrates Path 2: Direct API - Protobuf
// Flow: InternalResourceMetrics → Protobuf → export_metrics() → Arrow RecordBatch → Arrow IPC file
// No SDK! No temporary server!
//
// Alternative Import Paths:
// - Path 1: gRPC Protobuf Server (Port 4317)
//   Use opentelemetry-otlp exporter pointing to our gRPC server
//   Flow: SDK ResourceMetrics → opentelemetry-otlp (Protobuf) → Our server → Arrow RecordBatch → Arrow IPC file
//
// - Path 2: Direct API with Protobuf (this demo)
//   Create InternalResourceMetrics → to_protobuf() → library.export_metrics(protobuf_request)
//   Flow: InternalResourceMetrics → Protobuf → Arrow RecordBatch (internal) → Arrow IPC file
//
// All paths convert to Arrow RecordBatch as the internal format for storage.
// Note: Arrow ingestion for metrics is not supported - only Protobuf ingestion is available.

/// Generate mock spans with a counter for time-series demonstration
///
/// Similar to generate_mock_spans() but includes a counter parameter
/// to generate varying data over time, demonstrating time-series patterns
/// in the dashboard visualization.
///
/// # Arguments
///
/// * `counter` - Generation counter to create varying span attributes
///
/// # Returns
///
/// Vector of SpanData representing a complete trace with varying attributes
fn generate_mock_spans_with_counter(counter: u64) -> Vec<SpanData> {
    let mut spans = Vec::new();
    let now = SystemTime::now();

    // Create a trace ID that will be shared by all spans in this trace
    // Use counter to create unique trace IDs for each generation cycle
    // This demonstrates how different traces appear in the dashboard
    let mut trace_id_bytes = [1, 2, 3, 4, 5, 6, 7, 8, 9, 10, 11, 12, 13, 14, 15, 16];
    // Incorporate counter into trace ID to make each generation unique
    trace_id_bytes[0] = (counter % 256) as u8;
    trace_id_bytes[1] = ((counter / 256) % 256) as u8;
    let trace_id = TraceId::from_bytes(trace_id_bytes);

    // ============================================================================
    // Root Span: HTTP Server Request
    // ============================================================================

    // Root span has no parent (parent_span_id = SpanId::INVALID)
    let root_span_id = SpanId::from_bytes([1, 1, 1, 1, 1, 1, 1, 1]);
    let root_span_context = SpanContext::new(
        trace_id,
        root_span_id,
        TraceFlags::default(),
        false,
        TraceState::default(),
    );

    let root_span = SpanData {
        span_context: root_span_context,
        parent_span_id: SpanId::INVALID, // Root span has no parent
        span_kind: SpanKind::Server,     // Server span receives the request
        name: std::borrow::Cow::Borrowed("http-request"),
        start_time: now,
        end_time: now,
        attributes: vec![
            KeyValue::new("service.name", "demo-service"),
            KeyValue::new("http.method", "GET"),
            KeyValue::new("http.route", "/api/users"),
            KeyValue::new("http.status_code", 200),
        ]
        .into_iter()
        .collect(),
        events: opentelemetry_sdk::trace::SpanEvents::default(),
        links: opentelemetry_sdk::trace::SpanLinks::default(),
        status: Status::Ok,
        dropped_attributes_count: 0,
        parent_span_is_remote: false,
        instrumentation_scope: opentelemetry::InstrumentationScope::builder("demo-app")
            .with_version("1.0.0")
            .build(),
    };
    spans.push(root_span);

    // ============================================================================
    // Child Span 1: Database Query (Internal operation)
    // ============================================================================

    // Child span references parent via parent_span_id
    let db_span_id = SpanId::from_bytes([2, 2, 2, 2, 2, 2, 2, 2]);
    let db_span_context = SpanContext::new(
        trace_id, // Same trace_id as parent
        db_span_id,
        TraceFlags::default(),
        false,
        TraceState::default(),
    );

    let db_span = SpanData {
        span_context: db_span_context,
        parent_span_id: root_span_id,  // Reference to parent span
        span_kind: SpanKind::Internal, // Internal operation
        name: std::borrow::Cow::Borrowed("database-query"),
        start_time: now,
        end_time: now,
        attributes: vec![
            KeyValue::new("service.name", "demo-service"),
            KeyValue::new("db.system", "postgresql"),
            KeyValue::new("db.operation", "SELECT"),
            KeyValue::new("db.statement", "SELECT * FROM users WHERE id = ?"),
        ]
        .into_iter()
        .collect(),
        events: opentelemetry_sdk::trace::SpanEvents::default(),
        links: opentelemetry_sdk::trace::SpanLinks::default(),
        status: Status::Ok,
        dropped_attributes_count: 0,
        parent_span_is_remote: false,
        instrumentation_scope: opentelemetry::InstrumentationScope::builder("demo-app")
            .with_version("1.0.0")
            .build(),
    };
    spans.push(db_span);

    // ============================================================================
    // Child Span 2: External API Call (Client span)
    // ============================================================================

    let api_span_id = SpanId::from_bytes([3, 3, 3, 3, 3, 3, 3, 3]);
    let api_span_context = SpanContext::new(
        trace_id, // Same trace_id as parent
        api_span_id,
        TraceFlags::default(),
        false,
        TraceState::default(),
    );

    let api_span = SpanData {
        span_context: api_span_context,
        parent_span_id: root_span_id, // Reference to parent span
        span_kind: SpanKind::Client,  // Client span makes external call
        name: std::borrow::Cow::Borrowed("http-client-request"),
        start_time: now,
        end_time: now,
        attributes: vec![
            KeyValue::new("service.name", "demo-service"),
            KeyValue::new("http.method", "GET"),
            KeyValue::new("http.url", "https://api.example.com/data"),
            KeyValue::new("http.status_code", 200),
        ]
        .into_iter()
        .collect(),
        events: opentelemetry_sdk::trace::SpanEvents::default(),
        links: opentelemetry_sdk::trace::SpanLinks::default(),
        status: Status::Ok,
        dropped_attributes_count: 0,
        parent_span_is_remote: false,
        instrumentation_scope: opentelemetry::InstrumentationScope::builder("demo-app")
            .with_version("1.0.0")
            .build(),
    };
    spans.push(api_span);

    // ============================================================================
    // Additional Spans: Generate more spans to meet requirement (10+ spans)
    // ============================================================================

    // Generate additional spans with different patterns
    for i in 4..=12 {
        let span_id_bytes = [i as u8; 8];
        let span_id = SpanId::from_bytes(span_id_bytes);
        let span_context = SpanContext::new(
            trace_id,
            span_id,
            TraceFlags::default(),
            false,
            TraceState::default(),
        );

        // Alternate between different span kinds
        let (span_kind, name) = match i % 3 {
            0 => (SpanKind::Server, format!("http-server-{}", i)),
            1 => (SpanKind::Client, format!("http-client-{}", i)),
            _ => (SpanKind::Internal, format!("internal-operation-{}", i)),
        };

        let span = SpanData {
            span_context,
            parent_span_id: if i % 2 == 0 {
                root_span_id // Some are children of root
            } else {
                db_span_id // Some are children of db span
            },
            span_kind,
            name: std::borrow::Cow::Owned(name),
            start_time: now,
            end_time: now,
            attributes: vec![
                KeyValue::new("service.name", "demo-service"),
                KeyValue::new("operation.id", i),
                KeyValue::new("generation.counter", counter as i64), // Varying attribute for time-series
            ]
            .into_iter()
            .collect(),
            events: opentelemetry_sdk::trace::SpanEvents::default(),
            links: opentelemetry_sdk::trace::SpanLinks::default(),
            status: Status::Ok,
            dropped_attributes_count: 0,
            parent_span_is_remote: false,
            instrumentation_scope: opentelemetry::InstrumentationScope::builder("demo-app")
                .with_version("1.0.0")
                .build(),
        };
        spans.push(span);
    }

    spans
}
