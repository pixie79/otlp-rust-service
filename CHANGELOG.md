# Changelog

All notable changes to the OTLP Arrow Flight Library will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.0.0/),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

## [Unreleased]

## [0.3.0] - 2025-01-27

### Added
- **Built-in OpenTelemetry SDK Exporter Implementations**: Added `OtlpMetricExporter` and `OtlpSpanExporter` types that implement OpenTelemetry SDK's `PushMetricExporter` and `SpanExporter` traits, enabling direct integration with OpenTelemetry SDK without custom wrapper code
  - `OtlpLibrary::metric_exporter()` - Returns a `PushMetricExporter` implementation for use with `PeriodicReader` or `ManualReader`
  - `OtlpLibrary::span_exporter()` - Returns a `SpanExporter` implementation for use with `TracerProvider`
  - Both exporters delegate to the underlying `OtlpLibrary` instance and handle error conversion automatically
- **Reference-Based Metric Export**: Added `export_metrics_ref(&ResourceMetrics)` method to `OtlpLibrary` for efficient metric export without requiring ownership
  - Accepts `&ResourceMetrics` instead of owned `ResourceMetrics`, avoiding unnecessary data copying
  - Functionally equivalent to `export_metrics()` but more efficient for integration with OpenTelemetry SDK's periodic readers
  - Available in both Rust and Python APIs
- **Python Exporter Bindings**: Added Python bindings for exporter creation methods
  - `PyOtlpLibrary.metric_exporter()` - Returns `PyOtlpMetricExporter` instance
  - `PyOtlpLibrary.span_exporter()` - Returns `PyOtlpSpanExporter` instance
  - Foundation for future Python OpenTelemetry SDK integration (tracked in Issue #6)

## [0.2.0] - 2024-11-24

### Added
- **JavaScript/TypeScript/CSS Formatting and Linting**: Added comprehensive code quality checks for all languages in the codebase
  - Added Prettier configuration for formatting JavaScript, TypeScript, CSS, JSON, YAML, Markdown, and HTML files
  - Added ESLint configuration for linting JavaScript and TypeScript files
  - Updated constitution to require formatting and linting checks for all languages before commits
  - Enhanced CI pipeline to run JavaScript/TypeScript formatting and linting checks
  - Added npm scripts: `format`, `format:check`, `lint`, `lint:fix` to dashboard package.json
  - Fixed all ESLint errors in test files and benchmark scripts

### Changed

#### Web Realtime Dashboard
- **Web JS-based Realtime Dashboard**: Client-side web application for real-time visualization of OTLP traces and metrics
- **Live Trace Tail Viewer**: Real-time display of trace data as it arrives, with filtering and detail pane
- **Real-time Metrics Graphing**: Dynamic visualization of metric data over time using Plotly.js
- **Arrow IPC File Streaming**: Direct client-side reading of Arrow IPC files using File System Access API or FileReader API
- **DuckDB-wasm Integration**: Efficient in-browser querying of Arrow IPC files using DuckDB WebAssembly
- **Interactive Dashboard UI**: Responsive design with keyboard navigation, search, and accessibility features (WCAG 2.1 AA)
- **Rust Service Integration**: Optional HTTP server in Rust service to serve dashboard static files (disabled by default)
- **Configuration UI**: Settings panel for configuring polling interval, data limits, and memory management
- **Error Boundary**: Graceful error handling with user-friendly error messages
- **Browser Compatibility Detection**: Automatic detection and graceful degradation for unsupported browsers
- **Data Limits**: Configurable limits for traces, graph points, and loaded tables with automatic cleanup
- **State Persistence**: Dashboard state (view, pause state, time range) persisted in localStorage
- **Performance Optimizations**: Virtual scrolling for large trace lists, Plotly extendTraces for efficient graph updates, LRU cache for DuckDB tables
- **Comprehensive Testing**: Unit tests, integration tests, and E2E tests (Playwright) for all dashboard features

#### Dashboard Configuration
- `dashboard.enabled`: Enable/disable dashboard HTTP server (default: false)
- `dashboard.port`: Port for dashboard HTTP server (default: 8080)
- `dashboard.static_dir`: Directory containing dashboard static files (default: "./dashboard/dist")
- `dashboard.bind_address`: Bind address for dashboard HTTP server (default: "127.0.0.1" for local-only access, use "0.0.0.0" for network access)
- Environment variable overrides: `OTLP_DASHBOARD_ENABLED`, `OTLP_DASHBOARD_PORT`, `OTLP_DASHBOARD_STATIC_DIR`, `OTLP_DASHBOARD_BIND_ADDRESS`

## [0.1.0] - 2024-11-23

### Added

#### Core Features
- **Dual Protocol Support**: Simultaneous support for gRPC Protobuf (port 4317) and gRPC Arrow Flight (port 4318)
- **Arrow IPC Storage**: Writes OTLP traces and metrics to local files in Arrow IPC Streaming format
- **Batch Writing**: Configurable write intervals for efficient disk I/O
- **File Cleanup**: Automatic cleanup of old trace and metric files based on configurable retention intervals
- **Configuration System**: Support for YAML files, environment variables, and programmatic API
- **Public API**: Embedded library mode with programmatic methods for exporting traces and metrics

#### Remote Forwarding
- **Optional Forwarding**: Forward messages to remote OTLP endpoints
- **Format Conversion**: Automatic conversion between Protobuf and Arrow Flight formats
- **Authentication**: Support for API key, bearer token, and basic authentication
- **Circuit Breaker**: Automatic failure handling with circuit breaker pattern
- **Resilience**: Forwarding failures do not affect local storage

#### Testing & Development
- **Mock Service**: Full mock OTLP service for end-to-end testing
- **Comprehensive Tests**: Unit, integration, and contract tests
- **Python Bindings**: PyO3 bindings for Python integration
- **Performance Benchmarks**: Benchmarks for throughput, latency, and format conversion

#### Documentation & Examples
- **Comprehensive Documentation**: Full API documentation with examples
- **Usage Examples**: Standalone service, embedded library, and Python examples
- **Health Check**: HTTP health check endpoint for standalone service
- **Metrics Collection**: Library operation metrics (messages received, files written, errors, conversions)

### Configuration

- `output_dir`: Output directory for Arrow IPC files (default: `./output_dir`)
- `write_interval_secs`: How frequently to write batches to disk (default: 5 seconds)
- `trace_cleanup_interval_secs`: Trace file retention interval (default: 600 seconds)
- `metric_cleanup_interval_secs`: Metric file retention interval (default: 3600 seconds)
- `protocols`: Protocol configuration (Protobuf and Arrow Flight, both enabled by default)
- `forwarding`: Optional remote forwarding configuration

### API

#### Rust API
- `OtlpLibrary::new(config)` - Create library instance
- `OtlpLibrary::export_trace(span)` - Export single trace span
- `OtlpLibrary::export_traces(spans)` - Export multiple trace spans
- `OtlpLibrary::export_metrics(metrics)` - Export metrics
- `OtlpLibrary::flush()` - Force immediate flush
- `OtlpLibrary::shutdown()` - Graceful shutdown

#### Python API
- `PyOtlpLibrary()` - Create library instance
- `export_trace(span_dict)` - Export single trace from Python dict
- `export_traces(spans_list)` - Export multiple traces from Python list
- `export_metrics(metrics_dict)` - Export metrics from Python dict
- `flush()` - Force immediate flush
- `shutdown()` - Graceful shutdown

### Testing

- Unit tests for all core components
- Integration tests for end-to-end scenarios
- Contract tests for protocol compliance
- Mock service for testing without external dependencies
- Python bindings tests

### Performance

- Benchmarks for message throughput
- Benchmarks for operation latency
- Benchmarks for format conversion overhead

### Dependencies

- `opentelemetry` 0.31
- `opentelemetry-sdk` 0.31
- `opentelemetry-otlp` 0.31
- `opentelemetry-proto` 0.31
- `arrow` 57
- `arrow-array` 57
- `arrow-flight` 57
- `tokio` 1.35+
- `tonic` 0.14
- `pyo3` 0.20 (for Python bindings)

### Changed
- **Python Minimum Version**: Updated minimum supported Python version from 3.8 to 3.11
  - Updated `pyproject.toml` to require Python 3.11+
  - Updated specification, plan, and research documents to reflect Python 3.11 requirement

### Fixed

- **ResourceMetrics Clone Issue**: Fixed limitation where `ResourceMetrics` doesn't implement `Clone`, preventing proper buffering and forwarding
  - Metrics are now stored as `ExportMetricsServiceRequest` (protobuf) in the batch buffer, which implements `Clone`
  - `OtlpLibrary::export_metrics()` automatically converts `ResourceMetrics` to protobuf before storing
  - Flush operations convert protobuf back to `ResourceMetrics` when exporting to file
  - Arrow Flight metrics are converted to protobuf before storage
  - This ensures all metric data can be properly buffered, forwarded, and exported without data loss

### Known Limitations

- Arrow Flight to Protobuf conversion uses simplified implementation (full metadata reconstruction pending)
- ResourceMetrics to Protobuf conversion uses simplified implementation when converting from SDK `ResourceMetrics` (full metadata reconstruction pending)
  - Note: When metrics come from gRPC Protobuf, the original protobuf request is preserved, ensuring full data fidelity
- Arrow Flight forwarding client implementation is a placeholder (requires full gRPC client)

### Future Enhancements

- Full metadata reconstruction in format conversions
- Complete Arrow Flight forwarding client implementation
- Additional authentication methods
- Metrics aggregation and reporting
- Distributed tracing support

