# Data Model: Built-in OpenTelemetry Exporter Implementations

**Feature**: 003-opentelemetry-exporters  
**Date**: 2025-01-27

## Overview

This feature extends the existing `OtlpLibrary` data model with new exporter types and methods. No new storage or persistence entities are introduced - exporters delegate to existing `OtlpLibrary` functionality.

## Entities

### OtlpMetricExporter

**Purpose**: Wraps `OtlpLibrary` and implements `PushMetricExporter` trait for OpenTelemetry SDK integration.

**Fields**:
- `library: Arc<OtlpLibrary>` - Reference to the underlying library instance

**Relationships**:
- Wraps `OtlpLibrary` (one-to-one relationship)
- Used by OpenTelemetry SDK's `PeriodicReader` (one-to-many: one exporter can be used by multiple readers)

**State Transitions**:
- **Created**: Exporter created via `OtlpLibrary::metric_exporter()`
- **Active**: Exporter used by OpenTelemetry SDK to export metrics
- **Shutdown**: OpenTelemetry SDK calls `shutdown_with_timeout()` (does not affect library state)

**Validation Rules**:
- Library instance must be valid (not shut down) when exporter is created
- Exporter can be used after library shutdown (will fail on export, but exporter remains valid)

**Lifecycle**:
- Created: When `OtlpLibrary::metric_exporter()` is called
- Used: When OpenTelemetry SDK calls `export()` method
- Shutdown: When OpenTelemetry SDK calls `shutdown_with_timeout()` (no-op, library shutdown is separate)

---

### OtlpSpanExporter

**Purpose**: Wraps `OtlpLibrary` and implements `SpanExporter` trait for OpenTelemetry SDK integration.

**Fields**:
- `library: Arc<OtlpLibrary>` - Reference to the underlying library instance

**Relationships**:
- Wraps `OtlpLibrary` (one-to-one relationship)
- Used by OpenTelemetry SDK's `TracerProvider` (one-to-many: one exporter can be used by multiple providers)

**State Transitions**:
- **Created**: Exporter created via `OtlpLibrary::span_exporter()`
- **Active**: Exporter used by OpenTelemetry SDK to export spans
- **Shutdown**: OpenTelemetry SDK calls `shutdown()` (does not affect library state)

**Validation Rules**:
- Library instance must be valid (not shut down) when exporter is created
- Exporter can be used after library shutdown (will fail on export, but exporter remains valid)

**Lifecycle**:
- Created: When `OtlpLibrary::span_exporter()` is called
- Used: When OpenTelemetry SDK calls `export()` method
- Shutdown: When OpenTelemetry SDK calls `shutdown()` (no-op, library shutdown is separate)

---

### Reference-based Export Capability

**Purpose**: Enables efficient export of metrics by reference without requiring ownership.

**Fields**: N/A (capability, not an entity)

**Relationships**:
- Extends `OtlpLibrary` with `export_metrics_ref(&ResourceMetrics)` method
- Used by `OtlpMetricExporter::export()` to avoid cloning

**State**: N/A

**Validation Rules**:
- Reference must be valid (not null/dangling)
- Metrics data must be valid OpenTelemetry SDK `ResourceMetrics`
- Empty metrics are handled gracefully (no-op)

**Lifecycle**: N/A (method call, not an entity)

---

## Extended Entities

### OtlpLibrary (Extended)

**New Methods**:
- `export_metrics_ref(&self, metrics: &ResourceMetrics) -> Result<(), OtlpError>` - Export metrics by reference
- `metric_exporter(&self) -> OtlpMetricExporter` - Create metric exporter
- `span_exporter(&self) -> OtlpSpanExporter` - Create span exporter

**No Changes to Existing Fields**: All existing fields remain unchanged.

---

## Python Bindings Entities

### PyOtlpMetricExporter

**Purpose**: Python-compatible wrapper for `OtlpMetricExporter`.

**Fields**:
- `exporter: Arc<OtlpMetricExporter>` - Reference to Rust exporter

**Relationships**:
- Wraps `OtlpMetricExporter` (one-to-one)
- Created by `PyOtlpLibrary::metric_exporter()`

---

### PyOtlpSpanExporter

**Purpose**: Python-compatible wrapper for `OtlpSpanExporter`.

**Fields**:
- `exporter: Arc<OtlpSpanExporter>` - Reference to Rust exporter

**Relationships**:
- Wraps `OtlpSpanExporter` (one-to-one)
- Created by `PyOtlpLibrary::span_exporter()`

---

### PyOtlpLibrary (Extended)

**New Methods**:
- `metric_exporter(&self) -> PyResult<PyOtlpMetricExporter>` - Create Python metric exporter
- `span_exporter(&self) -> PyResult<PyOtlpSpanExporter>` - Create Python span exporter

**No Changes to Existing Fields**: All existing fields remain unchanged.

---

## Data Flow

### Metric Export Flow

1. OpenTelemetry SDK's `PeriodicReader` calls `OtlpMetricExporter::export(&ResourceMetrics)`
2. Exporter calls `OtlpLibrary::export_metrics_ref(&ResourceMetrics)`
3. Library converts `&ResourceMetrics` to protobuf via `FormatConverter::resource_metrics_to_protobuf()`
4. Protobuf request stored in `BatchBuffer`
5. Background task writes batch to disk at configured interval

### Span Export Flow

1. OpenTelemetry SDK's `TracerProvider` calls `OtlpSpanExporter::export(Vec<SpanData>)`
2. Exporter calls `OtlpLibrary::export_traces(Vec<SpanData>)`
3. Spans stored in `BatchBuffer`
4. Background task writes batch to disk at configured interval

---

## Constraints

- Exporters do not own `OtlpLibrary` - they share a reference via `Arc`
- Exporters do not manage library lifecycle - `OtlpLibrary::shutdown()` is called separately
- Exporters are thread-safe and can be used concurrently
- Reference-based export maintains functional equivalence with owned export
- Python bindings provide feature parity with Rust API

---

## Validation

- Exporter creation: Library must be valid (not None/null)
- Export operations: Library methods handle validation internally
- Error conversion: All `OtlpError` variants converted to `OTelSdkError::InternalFailure`
- Lifecycle: Exporters handle shutdown gracefully without affecting library

