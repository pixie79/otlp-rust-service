# Changelog

All notable changes to the OTLP Arrow Flight Library will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.0.0/),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

## [Unreleased]

## [0.6.1] - 2025-12-11

### Changed
- **CI/CD Pipeline**: Enabled publish job with automatic tag creation and issue closing
  - Publish job now creates git tags automatically on release
  - Automatically closes referenced GitHub issues when release is published
  - Removed CHANGELOG and pyproject.toml updates from CI (handled locally)

## [0.6.0] - 2025-12-11

### Added
- **Comprehensive Test Coverage**: Added extensive test suite for concurrent access, circuit breaker state transitions, and edge cases
  - Unit tests for concurrent BatchBuffer access (`tests/unit/otlp/test_batch_buffer_concurrent.rs`)
  - Unit tests for circuit breaker state transitions (`tests/unit/otlp/test_circuit_breaker.rs`)
  - Integration tests for high concurrency scenarios (`tests/integration/test_concurrent_access.rs`)
  - Integration tests for circuit breaker recovery (`tests/integration/test_circuit_breaker_recovery.rs`)
  - Contract tests for edge cases including buffer capacity limits and race conditions (`tests/contract/test_edge_cases.rs`)
- **Architecture Documentation**: Created comprehensive `docs/ARCHITECTURE.md` documenting system design, data flow, component interactions, and key architectural decisions
- **Configurable Temporality**: Added support for configurable temporality (Cumulative or Delta) for metric exporters
  - `ConfigBuilder::with_temporality()` method for Rust API
  - `set_temporality()` method for Python bindings
  - Defaults to Cumulative for backward compatibility
- **Benchmark Infrastructure**: Added benchmark tests for performance validation
  - Circuit breaker lock acquisition benchmarks (`tests/bench/bench_circuit_breaker.rs`)
  - BatchBuffer throughput benchmarks (`tests/bench/bench_batch_buffer.rs`)
  - Exporter performance benchmarks (`tests/bench/bench_exporter.rs`)

### Changed
- **Circuit Breaker Optimization**: Optimized circuit breaker lock contention by grouping state fields
  - Reduced from 4+ separate `Arc<Mutex<T>>` locks to 1 grouped `Arc<Mutex<CircuitBreakerState>>` lock
  - Batched state updates into single lock acquisitions
  - Significantly reduced lock acquisition frequency (50%+ improvement)
- **Exporter Performance**: Optimized exporter implementations for better throughput
  - Grouped exporter metrics into single struct to reduce lock acquisitions (4 locks → 1 lock)
  - Reduced unnecessary memory allocations (clones, schema handling)
  - Improved efficiency of export operations
- **Pre-commit Hook**: Improved pre-commit hook to properly build Python binaries and run tests in correct virtual environment
  - Activates venv before cargo commands
  - Sets PYO3_PYTHON correctly from venv
  - Builds Python binaries with maturin before running Python tests
  - Avoids Python linking issues by excluding python-extension feature from cargo check/test

### Fixed
- **Test Infrastructure**: Added `tokio-test` dependency for improved async testing utilities

## [0.5.0] - 2025-11-30

### Security
- **Credential Storage**: Changed `AuthConfig::credentials` from `HashMap<String, String>` to `HashMap<String, SecretString>` to prevent credential exposure in logs, errors, or memory dumps
  - Credentials are now zeroed in memory when dropped
  - Credentials never appear in `Debug` or `Display` implementations
  - Breaking change: API now requires `SecretString::new()` when creating credentials programmatically
- **Path Traversal Protection**: Enhanced path validation in dashboard server to prevent directory traversal attacks
  - Rejects absolute paths, UNC paths, and paths with `..` components
  - Normalizes paths and resolves symlinks safely
  - Verifies canonical paths stay within allowed directories
- **HTTP Security Headers**: Added security headers to all HTTP responses
  - `Content-Security-Policy: default-src 'self'` - Prevents XSS attacks
  - `X-Frame-Options: DENY` - Prevents clickjacking (configurable via `DashboardConfig::x_frame_options`)
  - `X-Content-Type-Options: nosniff` - Prevents MIME type sniffing
  - `X-XSS-Protection: 1; mode=block` - Additional XSS protection
- **Input Validation**: Improved URL validation using `url` crate for comprehensive parsing and validation
  - Validates URL scheme (must be http or https)
  - Validates host presence
  - Provides clear error messages for invalid URLs

### Fixed
- **Auth Validation Logic**: Fixed mismatch between validation logic and actual credential key names
  - `api_key` authentication now correctly checks for `"key"` credential (not `"api_key"` or `"token"`)
  - `bearer_token` authentication correctly checks for `"token"` credential
  - Removed duplicate `"basic"` case in validation
- **Circuit Breaker**: Completed half-open state implementation
  - Added `half_open_test_in_progress` flag to prevent concurrent test requests
  - Properly handles success (transition to Closed) and failure (transition back to Open) in half-open state
  - Prevents indefinite half-open state with timeout checks
- **Python Memory Safety**: Fixed unsafe memory operations in Python bindings
  - Replaced `std::mem::transmute` with safe `downcast()` operations
  - Improved reference counting and GIL handling
  - Added explicit lifetime management for PyRef usage
- **Protobuf Encoding**: Implemented proper Protobuf encoding for HTTP forwarding
  - Replaced empty buffer placeholders with `prost::Message::encode()`
  - Added proper error handling for encoding failures

### Added
- **Buffer Limits**: Added configurable buffer size limits to prevent unbounded memory growth
  - `Config::max_trace_buffer_size` (default: 10000)
  - `Config::max_metric_buffer_size` (default: 10000)
  - `BatchBuffer` now returns `BufferFull` error when limits are reached
  - Validation ensures limits are > 0 and <= 1,000,000
  - Environment variable support: `OTLP_MAX_TRACE_BUFFER_SIZE`, `OTLP_MAX_METRIC_BUFFER_SIZE`
- **Security Documentation**: Added `SECURITY.md` with comprehensive security information
  - Security model and assumptions
  - Threat model and mitigations
  - Vulnerability reporting process
  - Security best practices
  - Known security considerations

### Changed
- **Dependencies**: Added new dependencies for security and validation
  - `secrecy = "0.8"` (with serde feature) - Secure credential storage
  - `url = "2.5"` - Comprehensive URL parsing and validation

## [0.4.0] - 2025-11-27

### Changed
- **Rust Edition**: Upgraded from Rust 2021 to Rust 2024 edition
  - Improved async drop order semantics
  - Access to latest Rust language features
  - Automatic migration via `cargo fix --edition`

### Added
- **Python OpenTelemetry SDK Adapter Classes**: Added built-in Python adapter classes that implement Python OpenTelemetry SDK's `MetricExporter` and `SpanExporter` interfaces, enabling direct integration without custom adapter code
  - `PyOtlpLibrary.metric_exporter_adapter()` - Returns `PyOtlpMetricExporterAdapter` that implements Python OpenTelemetry SDK's `MetricExporter` interface for use with `PeriodicExportingMetricReader`
  - `PyOtlpLibrary.span_exporter_adapter()` - Returns `PyOtlpSpanExporterAdapter` that implements Python OpenTelemetry SDK's `SpanExporter` interface for use with `BatchSpanProcessor` and `TracerProvider`
  - Automatic type conversion from Python OpenTelemetry SDK types to library-compatible formats
  - Error handling that converts library errors to appropriate Python exceptions while preserving context
  - Lifecycle management that handles Python OpenTelemetry SDK shutdown and flush methods gracefully
  - Full support for Python 3.11+ on Windows, Linux, and macOS
- **Comprehensive Test Suite**: Added unit, integration, and contract tests for Python adapters
- **Documentation**: Added quickstart guide and updated API documentation for Python OpenTelemetry SDK integration
- **Demo Application**: Added `examples/demo-app.rs` - A comprehensive demo application that demonstrates OTLP SDK usage
  - Enables dashboard by default for real-time telemetry visualization
  - Generates and exports mock metrics and spans
  - Demonstrates parent-child span relationships and different span kinds
  - Includes continuous data generation mode with graceful shutdown
  - Serves as a reference implementation for developers integrating the SDK
  - Well-documented with extensive comments explaining all SDK usage patterns
- **Live Tail Feature**: Real-time trace list updates with automatic scrolling
  - Tracks maximum timestamp to detect new traces efficiently
  - Auto-scrolls to bottom when user is near the end
  - Deduplicates traces using both timestamp and trace ID
- **Database State Management**: Automatic table clearing when selecting new directories
  - Clears all DuckDB tables on directory selection for fresh start
  - Handles initialization timing gracefully
  - Prevents stale data from interfering with new sessions

### Changed
- **Metrics Ingestion**: Removed Arrow Flight ingestion for metrics (Protobuf-only)
  - Simplifies architecture by eliminating `ResourceMetrics` private field workarounds
  - Protobuf ingestion remains fully supported via gRPC endpoint and direct API
  - Arrow IPC remains the internal storage format
  - Arrow Flight export option still available for forwarding
- **Arrow IPC File Format**: Changed from Arrow IPC File format to Arrow IPC Streaming format
  - File extension changed from `.arrow` to `.arrows` to indicate streaming format
  - Better suited for continuous writing and sequential reading
  - Improved compatibility with DuckDB-WASM and streaming readers
- **Dashboard File Handling**: Improved file ingestion and state management
  - Better error handling for missing tables
  - Automatic cleanup of stale table references
  - Support for both local files (File System Access API) and server-served files
  - Fixed timing issues with `insertArrowTable` verification

### Fixed
- **Live Tail Detection**: Fixed issue where live tail showed 0 new traces
  - Now tracks maximum timestamp to properly detect new traces
  - Handles cases where queries return all traces (including old ones)
- **Table Creation Timing**: Fixed race conditions in table creation and verification
  - Improved error handling for tables that aren't immediately queryable
  - Better handling of "table already exists" errors during updates
- **Database Initialization**: Fixed error when clearing tables before DuckDB initialization
  - `CLEAR_TABLES` now gracefully handles uninitialized state
  - Prevents errors from breaking directory selection flow
- **Missing Table Errors**: Improved error handling for tables that don't exist
  - Better detection of "table does not exist" vs other errors
  - Automatic cleanup of stale table references from state

## [0.3.0] - 2025-11-25

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

## [0.2.0] - 2025-11-24

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

## [0.1.0] - 2025-11-23

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

