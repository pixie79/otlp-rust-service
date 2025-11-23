# Changelog

All notable changes to the OTLP Arrow Flight Library will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.0.0/),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

## [Unreleased]

### Changed
- **Python Minimum Version**: Updated minimum supported Python version from 3.8 to 3.11
  - Updated `pyproject.toml` to require Python 3.11+
  - Updated specification, plan, and research documents to reflect Python 3.11 requirement
  - Updated agent context configuration

### Documentation
- Updated specification with Python 3.11 clarification
- Updated implementation plan with Python 3.11 requirement
- Updated research document with Python 3.11 minimum version
- Updated tasks.md with Python 3.11 requirement in Phase 7

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

