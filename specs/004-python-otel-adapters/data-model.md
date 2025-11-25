# Data Model: Python OpenTelemetry SDK Adapter Classes

**Feature**: 004-python-otel-adapters  
**Date**: 2025-11-25

## Overview

This feature extends the existing Python bindings with adapter classes that implement Python OpenTelemetry SDK exporter interfaces. The adapters bridge between Python OpenTelemetry SDK types and the library's Python API, handling type conversion, error handling, and lifecycle management.

## Entities

### Python Metric Exporter Adapter

**Purpose**: Implements Python OpenTelemetry SDK's `MetricExporter` interface, enabling direct integration with `PeriodicExportingMetricReader`.

**Fields**:
- `library: Py<PyOtlpLibrary>` - Reference to the underlying Python `OtlpLibrary` instance (prevents garbage collection)
- `conversion_cache: Optional[Dict]` - Optional cache for type conversion results (performance optimization)

**Relationships**:
- Wraps `PyOtlpLibrary` (one-to-one relationship)
- Used by Python OpenTelemetry SDK's `PeriodicExportingMetricReader` (one-to-many: one adapter can be used by multiple readers)
- Delegates to `PyOtlpLibrary::export_metrics()` or `export_metrics_ref()` methods

**State Transitions**:
- **Created**: Adapter created via `PyOtlpLibrary::metric_exporter()` method
- **Active**: Adapter used by Python OpenTelemetry SDK to export metrics
- **Shutdown**: Python OpenTelemetry SDK calls `shutdown()` (no-op, library shutdown is separate)
- **Invalid**: Library instance has been shut down (export operations will fail)

**Validation Rules**:
- Library instance must be valid (not shut down) when adapter is created
- Adapter can be used after library shutdown (will fail on export, but adapter remains valid)
- Type conversion must preserve 100% of metric data without loss or corruption

**Lifecycle**:
- Created: When `PyOtlpLibrary::metric_exporter()` is called
- Used: When Python OpenTelemetry SDK calls `export()` method
- Shutdown: When Python OpenTelemetry SDK calls `shutdown()` (no-op, library shutdown is separate)
- Garbage Collected: When no Python references remain (library reference prevents premature GC)

**Methods** (Python OpenTelemetry SDK interface):
- `export(metrics_data: MetricExportResult) -> ExportResult`: Export metrics data
- `shutdown() -> None`: Shutdown the exporter (no-op)
- `force_flush(timeout_millis: Optional[int] = None) -> ExportResult`: Flush pending exports
- `temporality() -> Temporality`: Return temporality preference (default: CUMULATIVE)

---

### Python Span Exporter Adapter

**Purpose**: Implements Python OpenTelemetry SDK's `SpanExporter` interface, enabling direct integration with `BatchSpanProcessor` and `TracerProvider`.

**Fields**:
- `library: Py<PyOtlpLibrary>` - Reference to the underlying Python `OtlpLibrary` instance (prevents garbage collection)
- `conversion_cache: Optional[Dict]` - Optional cache for type conversion results (performance optimization)

**Relationships**:
- Wraps `PyOtlpLibrary` (one-to-one relationship)
- Used by Python OpenTelemetry SDK's `BatchSpanProcessor` and `TracerProvider` (one-to-many: one adapter can be used by multiple processors/providers)
- Delegates to `PyOtlpLibrary::export_traces()` method

**State Transitions**:
- **Created**: Adapter created via `PyOtlpLibrary::span_exporter()` method
- **Active**: Adapter used by Python OpenTelemetry SDK to export spans
- **Shutdown**: Python OpenTelemetry SDK calls `shutdown()` (no-op, library shutdown is separate)
- **Invalid**: Library instance has been shut down (export operations will fail)

**Validation Rules**:
- Library instance must be valid (not shut down) when adapter is created
- Adapter can be used after library shutdown (will fail on export, but adapter remains valid)
- Type conversion must preserve 100% of span data without loss or corruption

**Lifecycle**:
- Created: When `PyOtlpLibrary::span_exporter()` is called
- Used: When Python OpenTelemetry SDK calls `export()` method
- Shutdown: When Python OpenTelemetry SDK calls `shutdown()` (no-op, library shutdown is separate)
- Garbage Collected: When no Python references remain (library reference prevents premature GC)

**Methods** (Python OpenTelemetry SDK interface):
- `export(spans: Sequence[ReadableSpan]) -> SpanExportResult`: Export span data
- `shutdown() -> None`: Shutdown the exporter (no-op)
- `force_flush(timeout_millis: Optional[int] = None) -> SpanExportResult`: Flush pending exports

---

### Type Conversion Layer

**Purpose**: Converts between Python OpenTelemetry SDK types and library-compatible dictionary formats.

**Components**:

#### Metric Type Conversion

**Input**: Python OpenTelemetry SDK `MetricExportResult` (contains `ResourceMetrics`)
**Output**: Dictionary format compatible with `PyOtlpLibrary::export_metrics()` or `export_metrics_ref()`

**Conversion Steps**:
1. Extract resource attributes from `ResourceMetrics.resource`
2. Extract scope metrics from `ResourceMetrics.scope_metrics`
3. For each scope metric:
   - Extract instrumentation scope information
   - Extract metric data points (gauges, sums, histograms)
   - Convert metric data points to dictionary format
4. Build dictionary structure:
   ```python
   {
       "resource": {
           "attributes": {...},  # Resource attributes
       },
       "scope_metrics": [
           {
               "scope": {...},  # Instrumentation scope
               "metrics": [...]  # Metric data points
           }
       ]
   }
   ```

**Data Preservation**:
- All resource attributes preserved
- All instrumentation scope information preserved
- All metric data points preserved (name, description, unit, data points)
- All metric values preserved (gauge values, sum values, histogram buckets)

#### Span Type Conversion

**Input**: Python OpenTelemetry SDK `Sequence[ReadableSpan]`
**Output**: List of dictionaries compatible with `PyOtlpLibrary::export_traces()`

**Conversion Steps**:
1. For each `ReadableSpan`:
   - Extract trace_id (16 bytes)
   - Extract span_id (8 bytes)
   - Extract parent_span_id (8 bytes, optional)
   - Extract name, kind, status
   - Extract attributes, events, links
   - Extract timestamps (start_time, end_time)
2. Build dictionary structure (follows existing `dict_to_span_data` pattern):
   ```python
   {
       "trace_id": bytes(...),  # 16 bytes
       "span_id": bytes(...),   # 8 bytes
       "parent_span_id": bytes(...),  # 8 bytes, optional
       "name": "...",
       "kind": "...",  # "server", "client", "internal", etc.
       "attributes": {...},
       "events": [...],
       "links": [...],
       "status": "...",
       "start_time": ...,
       "end_time": ...
   }
   ```

**Data Preservation**:
- All span identifiers preserved (trace_id, span_id, parent_span_id)
- All span metadata preserved (name, kind, status)
- All attributes preserved (key-value pairs)
- All events preserved (with timestamps and attributes)
- All links preserved (with trace_id, span_id, attributes)
- All timestamps preserved (start_time, end_time, event times)

#### Error Conversion

**Input**: Rust `OtlpError` or Python exceptions from library
**Output**: Python OpenTelemetry SDK `ExportResult` or `SpanExportResult`

**Conversion Rules**:
- Library errors → `ExportResult.FAILURE` or `SpanExportResult.FAILURE`
- Error context preserved in exception messages
- Appropriate Python exception types used (`RuntimeError`, `ValueError`)

---

## Relationships

```
PyOtlpLibrary
    ├── metric_exporter() → PyOtlpMetricExporterAdapter
    │                           ├── wraps: PyOtlpLibrary
    │                           ├── implements: Python OpenTelemetry SDK MetricExporter
    │                           └── uses: Type Conversion Layer
    │
    └── span_exporter() → PyOtlpSpanExporterAdapter
                            ├── wraps: PyOtlpLibrary
                            ├── implements: Python OpenTelemetry SDK SpanExporter
                            └── uses: Type Conversion Layer

Type Conversion Layer
    ├── converts: Python OpenTelemetry SDK MetricExportResult → Dictionary
    └── converts: Python OpenTelemetry SDK ReadableSpan → Dictionary
```

## Data Flow

### Metric Export Flow

1. Python OpenTelemetry SDK calls `adapter.export(metrics_data)`
2. Adapter receives `MetricExportResult` from Python OpenTelemetry SDK
3. Type Conversion Layer converts `MetricExportResult` → Dictionary
4. Adapter calls `library.export_metrics_ref(dictionary)` or `library.export_metrics(dictionary)`
5. Library processes and stores metrics
6. Adapter returns `ExportResult.SUCCESS` or `ExportResult.FAILURE`

### Span Export Flow

1. Python OpenTelemetry SDK calls `adapter.export(spans)`
2. Adapter receives `Sequence[ReadableSpan]` from Python OpenTelemetry SDK
3. Type Conversion Layer converts each `ReadableSpan` → Dictionary
4. Adapter calls `library.export_traces(list_of_dictionaries)`
5. Library processes and stores spans
6. Adapter returns `SpanExportResult.SUCCESS` or `SpanExportResult.FAILURE`

## Constraints

- Type conversion must preserve 100% of data (no loss or corruption)
- Adapters must remain valid while in use by Python OpenTelemetry SDK (garbage collection handling)
- Error context must be preserved during error conversion
- Concurrent use from multiple Python OpenTelemetry SDK components must be supported
- Cross-platform compatibility (Windows, Linux, macOS) must be maintained

