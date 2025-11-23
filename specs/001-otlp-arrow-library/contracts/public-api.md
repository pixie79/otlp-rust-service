# Public API Contract: OTLP Arrow Flight Library

**Date**: 2024-11-23  
**Feature**: 001-otlp-arrow-library

## Overview

The library provides a public API for embedded usage, allowing applications to programmatically send OTLP messages without using the gRPC interface. The API is available in two forms:
- **Rust API**: Native Rust API for direct use in Rust projects
- **Python API**: Python bindings (via PyO3) for use in Python projects

Both APIs provide equivalent functionality and follow the same patterns for consistency.

## Public API Methods

### Library Initialization

#### `OtlpLibrary::new(config: Config) -> Result<Self>`

Creates a new OTLP library instance with the provided configuration.

**Parameters**:
- `config: Config` - Configuration object (see data-model.md)

**Returns**: `Result<OtlpLibrary>` - Library instance or error

**Errors**:
- `ConfigError` - Invalid configuration values
- `IoError` - Failed to create output directory

**Example**:
```rust
use otlp_arrow_library::{OtlpLibrary, Config};

let config = Config::default();
let library = OtlpLibrary::new(config)?;
```

---

#### `OtlpLibrary::with_config_builder() -> ConfigBuilder`

Creates a configuration builder for programmatic configuration.

**Returns**: `ConfigBuilder` - Configuration builder

**Example**:
```rust
let library = OtlpLibrary::with_config_builder()
    .output_dir("./custom_output")
    .write_interval_secs(10)
    .build()?;
```

---

### Trace Operations

#### `OtlpLibrary::export_trace(&self, span: SpanData) -> Result<()>`

Exports a single trace span to the library.

**Parameters**:
- `span: SpanData` - OpenTelemetry span data

**Returns**: `Result<()>` - Success or error

**Errors**:
- `ExportError` - Failed to buffer or process span
- `IoError` - File system error

**Example**:
```rust
use opentelemetry_sdk::trace::SpanData;

let span: SpanData = /* create span */;
library.export_trace(span)?;
```

---

#### `OtlpLibrary::export_traces(&self, spans: Vec<SpanData>) -> Result<()>`

Exports multiple trace spans in a single call.

**Parameters**:
- `spans: Vec<SpanData>` - Collection of span data

**Returns**: `Result<()>` - Success or error

**Errors**:
- `ExportError` - Failed to buffer or process spans
- `IoError` - File system error

**Example**:
```rust
let spans: Vec<SpanData> = vec![/* spans */];
library.export_traces(spans)?;
```

---

### Metric Operations

#### `OtlpLibrary::export_metrics(&self, metrics: ResourceMetrics) -> Result<()>`

Exports metrics data to the library.

**Parameters**:
- `metrics: ResourceMetrics` - OpenTelemetry resource-scoped metrics

**Returns**: `Result<()>` - Success or error

**Errors**:
- `ExportError` - Failed to buffer or process metrics
- `IoError` - File system error

**Example**:
```rust
use opentelemetry_sdk::metrics::data::ResourceMetrics;

let metrics: ResourceMetrics = /* create metrics */;
library.export_metrics(metrics)?;
```

---

### Service Lifecycle

#### `OtlpLibrary::start_grpc_server(&self, address: SocketAddr) -> Result<()>`

Starts the gRPC server for receiving OTLP messages (standalone mode).

**Parameters**:
- `address: SocketAddr` - Address to bind the gRPC server

**Returns**: `Result<()>` - Success or error

**Errors**:
- `ServerError` - Failed to start gRPC server
- `IoError` - Network binding error

**Example**:
```rust
use std::net::SocketAddr;

let addr: SocketAddr = "0.0.0.0:4317".parse()?;
library.start_grpc_server(addr)?;
```

---

#### `OtlpLibrary::shutdown(&self) -> Result<()>`

Gracefully shuts down the library, flushing all pending writes.

**Returns**: `Result<()>` - Success or error

**Errors**:
- `ShutdownError` - Failed to flush or shutdown cleanly

**Example**:
```rust
library.shutdown()?;
```

---

#### `OtlpLibrary::flush(&self) -> Result<()>`

Forces immediate flush of all buffered messages to disk.

**Returns**: `Result<()>` - Success or error

**Errors**:
- `IoError` - File system error during flush

**Example**:
```rust
library.flush()?;
```

---

## Error Types

### `OtlpError`

Base error type for all library errors.

**Variants**:
- `Config(OtlpConfigError)` - Configuration errors
- `Export(OtlpExportError)` - Export/processing errors
- `Io(std::io::Error)` - I/O errors
- `Server(OtlpServerError)` - Server errors

---

### `OtlpConfigError`

Configuration-related errors.

**Variants**:
- `InvalidOutputDir(String)` - Invalid output directory path
- `InvalidInterval(String)` - Invalid interval value
- `MissingRequiredField(String)` - Missing required configuration field

---

### `OtlpExportError`

Export/processing errors.

**Variants**:
- `BufferFull` - Message buffer is full
- `SerializationError(String)` - Failed to serialize message
- `ForwardingError(String)` - Remote forwarding failed

---

### `OtlpServerError`

Server-related errors.

**Variants**:
- `BindError(String)` - Failed to bind server address
- `StartupError(String)` - Failed to start server

---

## Thread Safety

All public API methods are thread-safe and can be called from multiple threads concurrently. The library uses internal synchronization (Arc, Mutex, RwLock) to ensure safe concurrent access.

---

## Async vs Sync

The library provides both synchronous and asynchronous APIs:

- **Synchronous**: Methods return `Result<T>` directly (blocking)
- **Asynchronous**: Methods return `Future<Output = Result<T>>` (non-blocking)

Async methods are preferred for high-throughput scenarios.

---

## Configuration API

### `Config`

Main configuration structure (see data-model.md for fields).

### `ConfigBuilder`

Builder pattern for creating configurations.

**Methods**:
- `output_dir(path: impl Into<PathBuf>) -> Self`
- `write_interval_secs(secs: u64) -> Self`
- `trace_cleanup_interval_secs(secs: u64) -> Self`
- `metric_cleanup_interval_secs(secs: u64) -> Self`
- `enable_forwarding(config: ForwardingConfig) -> Self`
- `build() -> Result<Config>`

---

## Mock Service API

### `MockOtlpService::new() -> Self`

Creates a new mock OTLP service for testing.

### `MockOtlpService::receive_trace(&self, span: SpanData)`

Simulates receiving a trace via public API.

### `MockOtlpService::receive_metric(&self, metrics: ResourceMetrics)`

Simulates receiving metrics via public API.

### `MockOtlpService::assert_traces_received(&self, count: usize) -> Result<()>`

Asserts that the expected number of traces were received.

### `MockOtlpService::assert_metrics_received(&self, count: usize) -> Result<()>`

Asserts that the expected number of metrics were received.

### `MockOtlpService::reset(&self)`

Resets the mock service state (for test isolation).

---

## Versioning

The public API follows semantic versioning:
- **MAJOR**: Breaking changes to public API
- **MINOR**: New features, backward compatible
- **PATCH**: Bug fixes, backward compatible

Breaking changes will be documented in CHANGELOG.md with migration guides.

