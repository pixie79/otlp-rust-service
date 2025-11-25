# Rust API Contract: Built-in OpenTelemetry Exporter Implementations

**Date**: 2025-01-27  
**Feature**: 003-opentelemetry-exporters

## Overview

This contract defines the Rust API extensions for built-in OpenTelemetry SDK exporter implementations. These extensions enable seamless integration with OpenTelemetry SDK without requiring custom wrapper code.

## Extended API Methods

### OtlpLibrary Extensions

#### `OtlpLibrary::export_metrics_ref(&self, metrics: &ResourceMetrics) -> Result<(), OtlpError>`

Exports metrics by reference, avoiding unnecessary data copying when integrating with OpenTelemetry SDK's periodic readers.

**Parameters**:
- `metrics: &ResourceMetrics` - Reference to OpenTelemetry SDK resource metrics

**Returns**: `Result<(), OtlpError>` - Success or error

**Errors**:
- `OtlpExportError` - Failed to convert or buffer metrics
- `OtlpError::Io` - File system error

**Behavior**:
- Converts `&ResourceMetrics` to protobuf format (same as `export_metrics`)
- Stores protobuf in batch buffer
- Functionally equivalent to `export_metrics(ResourceMetrics)` but more efficient

**Example**:
```rust
use otlp_arrow_library::OtlpLibrary;
use opentelemetry_sdk::metrics::data::ResourceMetrics;

let library = OtlpLibrary::new(config).await?;
let metrics: ResourceMetrics = /* ... */;
library.export_metrics_ref(&metrics).await?;
```

---

#### `OtlpLibrary::metric_exporter(&self) -> OtlpMetricExporter`

Creates a `PushMetricExporter` implementation that can be used directly with OpenTelemetry SDK's `PeriodicReader`.

**Returns**: `OtlpMetricExporter` - Ready-to-use metric exporter

**Example**:
```rust
use otlp_arrow_library::OtlpLibrary;
use opentelemetry_sdk::metrics::PeriodicReader;

let library = OtlpLibrary::new(config).await?;
let metric_exporter = library.metric_exporter();
let reader = PeriodicReader::builder(metric_exporter)
    .with_interval(std::time::Duration::from_secs(10))
    .build();
```

---

#### `OtlpLibrary::span_exporter(&self) -> OtlpSpanExporter`

Creates a `SpanExporter` implementation that can be used directly with OpenTelemetry SDK's `TracerProvider`.

**Returns**: `OtlpSpanExporter` - Ready-to-use span exporter

**Example**:
```rust
use otlp_arrow_library::OtlpLibrary;
use opentelemetry_sdk::trace::TracerProvider;

let library = OtlpLibrary::new(config).await?;
let span_exporter = library.span_exporter();
let provider = TracerProvider::builder()
    .with_batch_exporter(span_exporter, opentelemetry_sdk::runtime::Tokio)
    .build();
```

---

## New Types

### OtlpMetricExporter

**Type**: `pub struct OtlpMetricExporter`

**Trait Implementation**: `opentelemetry_sdk::metrics::exporter::PushMetricExporter`

**Fields**: Private (wraps `Arc<OtlpLibrary>`)

**Methods** (via trait implementation):

- `export(&self, metrics: &ResourceMetrics) -> impl Future<Output = OTelSdkResult> + Send`
  - Delegates to `OtlpLibrary::export_metrics_ref()`
  - Converts `OtlpError` to `OTelSdkError::InternalFailure`

- `force_flush(&self) -> OTelSdkResult`
  - Calls `OtlpLibrary::flush()` if runtime handle is available
  - Returns `Ok(())` if no runtime handle (non-blocking)

- `shutdown_with_timeout(&self, timeout: Duration) -> OTelSdkResult`
  - Returns `Ok(())` immediately
  - Library shutdown is handled separately via `OtlpLibrary::shutdown()`

- `temporality(&self) -> Temporality`
  - Returns `Temporality::Cumulative` (default)

**Usage**:
```rust
let exporter = library.metric_exporter();
// Use with OpenTelemetry SDK PeriodicReader
```

---

### OtlpSpanExporter

**Type**: `pub struct OtlpSpanExporter`

**Trait Implementation**: `opentelemetry_sdk::trace::SpanExporter`

**Fields**: Private (wraps `Arc<OtlpLibrary>`)

**Methods** (via trait implementation):

- `export(&self, batch: Vec<SpanData>) -> BoxFuture<'static, OTelSdkResult>`
  - Delegates to `OtlpLibrary::export_traces()`
  - Converts `OtlpError` to `OTelSdkError::InternalFailure`

- `shutdown(&mut self) -> OTelSdkResult`
  - Returns `Ok(())` immediately
  - Library shutdown is handled separately via `OtlpLibrary::shutdown()`

**Usage**:
```rust
let exporter = library.span_exporter();
// Use with OpenTelemetry SDK TracerProvider
```

---

## Error Handling

### Error Conversion

All `OtlpError` variants are converted to `OTelSdkError::InternalFailure` with error message:

```rust
OTelSdkError::InternalFailure(format!("OtlpLibrary export failed: {}", e))
```

**Error Context Preservation**:
- Original error message included in converted error
- Errors logged via `tracing::warn!` before conversion
- Error type information preserved in message

---

## Lifecycle Management

### Exporter Lifecycle

- **Creation**: Exporters created via `OtlpLibrary` convenience methods
- **Usage**: Exporters used by OpenTelemetry SDK components
- **Shutdown**: Exporters handle shutdown gracefully but do not shut down library

### Library Lifecycle

- **Independent**: Library lifecycle is independent of exporter lifecycle
- **Multiple Exporters**: Multiple exporters can share the same library instance
- **Shutdown**: Developers call `OtlpLibrary::shutdown()` separately when application shuts down

---

## Thread Safety

- Exporters are `Send + Sync` (wrap `Arc<OtlpLibrary>`)
- Can be used concurrently from multiple OpenTelemetry SDK components
- `OtlpLibrary` methods are already thread-safe

---

## Integration Examples

### Complete Metric Integration

```rust
use otlp_arrow_library::OtlpLibrary;
use opentelemetry_sdk::metrics::{MeterProvider, PeriodicReader};

let library = OtlpLibrary::new(config).await?;
let metric_exporter = library.metric_exporter();
let reader = PeriodicReader::builder(metric_exporter)
    .with_interval(std::time::Duration::from_secs(10))
    .build();

let provider = MeterProvider::builder()
    .with_reader(reader)
    .build();

// Use provider to create meters and record metrics
// Metrics are automatically exported via the exporter
```

### Complete Trace Integration

```rust
use otlp_arrow_library::OtlpLibrary;
use opentelemetry_sdk::trace::TracerProvider;

let library = OtlpLibrary::new(config).await?;
let span_exporter = library.span_exporter();
let provider = TracerProvider::builder()
    .with_batch_exporter(span_exporter, opentelemetry_sdk::runtime::Tokio)
    .build();

// Use provider to create tracers and record spans
// Spans are automatically exported via the exporter
```

---

## Backward Compatibility

- All existing `OtlpLibrary` methods remain unchanged
- New methods are additive (no breaking changes)
- Existing code continues to work without modification
- `export_metrics_ref` is additional method (does not replace `export_metrics`)

