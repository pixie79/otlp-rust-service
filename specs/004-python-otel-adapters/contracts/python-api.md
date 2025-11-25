# Python API Contract: Python OpenTelemetry SDK Adapter Classes

**Date**: 2025-11-25  
**Feature**: 004-python-otel-adapters

## Overview

This contract defines the Python API for adapter classes that implement Python OpenTelemetry SDK exporter interfaces. These adapters enable seamless integration between Python OpenTelemetry SDK and `OtlpLibrary` without requiring custom adapter code.

## Extended API Methods

### PyOtlpLibrary Extensions

#### `PyOtlpLibrary.metric_exporter() -> PyOtlpMetricExporterAdapter`

Creates a Python metric exporter adapter that implements Python OpenTelemetry SDK's `MetricExporter` interface.

**Returns**: `PyOtlpMetricExporterAdapter` - Python metric exporter adapter implementing `opentelemetry.sdk.metrics.export.MetricExporter`

**Raises**:
- `RuntimeError` - If library instance is invalid or adapter creation fails

**Example**:
```python
import otlp_arrow_library
from opentelemetry.sdk.metrics.export import PeriodicExportingMetricReader

library = otlp_arrow_library.PyOtlpLibrary(output_dir="/tmp/otlp")
metric_exporter = library.metric_exporter()

# Use directly with Python OpenTelemetry SDK
reader = PeriodicExportingMetricReader(metric_exporter, export_interval_millis=5000)
```

**Interface Compliance**:
- Implements `opentelemetry.sdk.metrics.export.MetricExporter` abstract base class
- Compatible with `PeriodicExportingMetricReader` and `ManualReader`

---

#### `PyOtlpLibrary.span_exporter() -> PyOtlpSpanExporterAdapter`

Creates a Python span exporter adapter that implements Python OpenTelemetry SDK's `SpanExporter` interface.

**Returns**: `PyOtlpSpanExporterAdapter` - Python span exporter adapter implementing `opentelemetry.sdk.trace.export.SpanExporter`

**Raises**:
- `RuntimeError` - If library instance is invalid or adapter creation fails

**Example**:
```python
import otlp_arrow_library
from opentelemetry.sdk.trace.export import BatchSpanProcessor

library = otlp_arrow_library.PyOtlpLibrary(output_dir="/tmp/otlp")
span_exporter = library.span_exporter()

# Use directly with Python OpenTelemetry SDK
processor = BatchSpanProcessor(span_exporter)
tracer_provider.add_span_processor(processor)
```

**Interface Compliance**:
- Implements `opentelemetry.sdk.trace.export.SpanExporter` abstract base class
- Compatible with `BatchSpanProcessor` and `SimpleSpanProcessor`

---

## New Types

### PyOtlpMetricExporterAdapter

**Type**: Python class implementing `opentelemetry.sdk.metrics.export.MetricExporter`

**Purpose**: Adapter that bridges Python OpenTelemetry SDK's metric export system with `OtlpLibrary`

**Interface Methods**:

#### `export(metrics_data: MetricExportResult) -> ExportResult`

Exports metrics data to the library.

**Parameters**:
- `metrics_data: MetricExportResult` - Metrics data from Python OpenTelemetry SDK

**Returns**: `ExportResult` - Export result (`SUCCESS` or `FAILURE`)

**Raises**:
- `RuntimeError` - If export fails (library error, type conversion error)

**Behavior**:
- Converts Python OpenTelemetry SDK metric types to library-compatible dictionary format
- Delegates to `library.export_metrics_ref()` or `library.export_metrics()`
- Returns `ExportResult.SUCCESS` on successful export
- Returns `ExportResult.FAILURE` on error (error context preserved in exception)

---

#### `shutdown() -> None`

Shuts down the exporter. This is a no-op operation as library shutdown is handled separately.

**Returns**: `None`

**Behavior**:
- Does not shut down the underlying library
- Library shutdown remains a separate operation (`library.shutdown()`)
- Called by Python OpenTelemetry SDK during SDK shutdown

---

#### `force_flush(timeout_millis: Optional[int] = None) -> ExportResult`

Forces immediate flush of all pending exports.

**Parameters**:
- `timeout_millis: Optional[int]` - Timeout in milliseconds (optional, ignored)

**Returns**: `ExportResult` - Flush result (`SUCCESS` or `FAILURE`)

**Raises**:
- `RuntimeError` - If flush fails

**Behavior**:
- Delegates to `library.flush()`
- Returns `ExportResult.SUCCESS` on successful flush
- Returns `ExportResult.FAILURE` on error

---

#### `temporality() -> Temporality`

Returns the temporality preference for metrics.

**Returns**: `Temporality` - Temporality enum value (default: `CUMULATIVE`)

**Behavior**:
- Returns `Temporality.CUMULATIVE` by default
- Can be extended in future to support configurable temporality

---

### PyOtlpSpanExporterAdapter

**Type**: Python class implementing `opentelemetry.sdk.trace.export.SpanExporter`

**Purpose**: Adapter that bridges Python OpenTelemetry SDK's trace export system with `OtlpLibrary`

**Interface Methods**:

#### `export(spans: Sequence[ReadableSpan]) -> SpanExportResult`

Exports span data to the library.

**Parameters**:
- `spans: Sequence[ReadableSpan]` - Sequence of spans from Python OpenTelemetry SDK

**Returns**: `SpanExportResult` - Export result (`SUCCESS` or `FAILURE`)

**Raises**:
- `RuntimeError` - If export fails (library error, type conversion error)

**Behavior**:
- Converts Python OpenTelemetry SDK span types to library-compatible dictionary format
- Delegates to `library.export_traces()`
- Returns `SpanExportResult.SUCCESS` on successful export
- Returns `SpanExportResult.FAILURE` on error (error context preserved in exception)

---

#### `shutdown() -> None`

Shuts down the exporter. This is a no-op operation as library shutdown is handled separately.

**Returns**: `None`

**Behavior**:
- Does not shut down the underlying library
- Library shutdown remains a separate operation (`library.shutdown()`)
- Called by Python OpenTelemetry SDK during SDK shutdown

---

#### `force_flush(timeout_millis: Optional[int] = None) -> SpanExportResult`

Forces immediate flush of all pending exports.

**Parameters**:
- `timeout_millis: Optional[int]` - Timeout in milliseconds (optional, ignored)

**Returns**: `SpanExportResult` - Flush result (`SUCCESS` or `FAILURE`)

**Raises**:
- `RuntimeError` - If flush fails

**Behavior**:
- Delegates to `library.flush()`
- Returns `SpanExportResult.SUCCESS` on successful flush
- Returns `SpanExportResult.FAILURE` on error

---

## Error Handling

### Error Types

- `RuntimeError`: Used for library errors, type conversion errors, and adapter failures
- `ValueError`: Used for invalid input data (if applicable)

### Error Context Preservation

- Error messages include original error details
- Error context from library errors is preserved in exception messages
- Stack traces are preserved for debugging

### Error Conversion

- Library errors (`OtlpError`) → `RuntimeError` with error message
- Type conversion errors → `RuntimeError` with conversion details
- Export failures → `ExportResult.FAILURE` or `SpanExportResult.FAILURE`

## Lifecycle Management

### Adapter Creation

- Adapters are created via `library.metric_exporter()` or `library.span_exporter()`
- Adapters hold a reference to the library to prevent garbage collection
- Adapters remain valid while library instance is valid

### Adapter Shutdown

- Adapter `shutdown()` methods are no-ops (library shutdown is separate)
- Adapters can be used after library shutdown (export operations will fail)
- Python OpenTelemetry SDK manages adapter lifecycle

### Garbage Collection

- Adapters hold `Py<PyOtlpLibrary>` references to prevent premature garbage collection
- Adapters remain valid while in use by Python OpenTelemetry SDK
- No memory leaks or invalid object references

## Thread Safety

- Adapters support concurrent use from multiple Python OpenTelemetry SDK components
- Underlying library is thread-safe (uses `Arc` internally)
- Type conversion is thread-safe (no shared mutable state)

## Platform Compatibility

- Windows: Fully supported
- Linux: Fully supported
- macOS: Fully supported
- Python 3.11+: Required

## Version Compatibility

- Python OpenTelemetry SDK: Current stable version (tested with latest)
- Adapters target stable Python OpenTelemetry SDK interfaces
- Version compatibility can be extended in future enhancements

