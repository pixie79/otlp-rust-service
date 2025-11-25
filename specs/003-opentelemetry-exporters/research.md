# Research: Built-in OpenTelemetry Exporter Implementations

**Feature**: 003-opentelemetry-exporters  
**Date**: 2025-01-27  
**Status**: Complete

## Research Questions

### 1. ResourceMetrics Clone Limitation

**Question**: Why doesn't `ResourceMetrics` implement `Clone`, and how should we handle reference-based export?

**Research Findings**:

- **Decision**: `ResourceMetrics` from `opentelemetry-sdk` 0.31 has private fields and does not implement `Clone`
- **Rationale**: 
  - OpenTelemetry SDK intentionally keeps `ResourceMetrics` fields private to maintain API stability
  - The type is designed to be passed by reference in OpenTelemetry SDK's periodic readers
  - Cloning would require extracting all internal data, which is inefficient

**Implementation Approach**:
- Create `export_metrics_ref(&ResourceMetrics)` method that accepts a reference
- Convert `&ResourceMetrics` to protobuf format (same as existing `export_metrics` method)
- Use `FormatConverter::resource_metrics_to_protobuf(&metrics)` which already accepts a reference
- Store protobuf in batch buffer (same path as owned export)

**Alternatives Considered**:
- **Clone workaround**: Attempt to extract and clone all fields - REJECTED: Fields are private, not feasible
- **Owned export only**: Keep only `export_metrics(ResourceMetrics)` - REJECTED: Inefficient for OpenTelemetry SDK integration
- **Arc wrapper**: Wrap ResourceMetrics in Arc - REJECTED: Doesn't solve the problem, OpenTelemetry SDK passes by reference

**References**:
- Existing code: `src/otlp/converter.rs::resource_metrics_to_protobuf(&ResourceMetrics)`
- Existing code: `src/api/public.rs::export_metrics(ResourceMetrics)` - converts to protobuf then stores

---

### 2. OpenTelemetry SDK Trait Implementation Patterns

**Question**: How should we implement `PushMetricExporter` and `SpanExporter` traits to integrate with OpenTelemetry SDK?

**Research Findings**:

- **Decision**: Implement traits directly on new types (`OtlpMetricExporter`, `OtlpSpanExporter`) that wrap `Arc<OtlpLibrary>`
- **Rationale**:
  - Follows existing pattern: `FileMetricExporter` and `FileSpanExporter` already implement these traits
  - `Arc<OtlpLibrary>` allows sharing across async tasks and threads
  - Delegation pattern keeps implementation simple and maintainable

**Implementation Approach**:
- Create `OtlpMetricExporter` struct with `library: Arc<OtlpLibrary>` field
- Implement `PushMetricExporter` trait:
  - `export(&self, metrics: &ResourceMetrics)` - calls `library.export_metrics_ref(metrics)`
  - `force_flush(&self)` - calls `library.flush()` (async, needs runtime handle)
  - `shutdown_with_timeout(&self, timeout)` - returns Ok (library shutdown is separate)
  - `temporality(&self)` - returns `Temporality::Cumulative` (default)
- Create `OtlpSpanExporter` struct with `library: Arc<OtlpLibrary>` field
- Implement `SpanExporter` trait:
  - `export(&self, batch: Vec<SpanData>)` - calls `library.export_traces(batch)`
  - `shutdown(&mut self)` - returns Ok (library shutdown is separate)

**Error Handling**:
- Convert `OtlpError` to `OTelSdkError::InternalFailure` with error message
- Preserve error context in error message string
- Log errors using `tracing::warn!` before converting

**Alternatives Considered**:
- **Direct implementation on OtlpLibrary**: REJECTED - Would require `&mut self` for `SpanExporter::shutdown`, breaks API
- **Separate crate**: REJECTED - Adds complexity, version management overhead
- **Macro-based generation**: REJECTED - Over-engineering for simple delegation

**References**:
- Existing implementations: `src/otlp/exporter.rs::FileMetricExporter`, `FileSpanExporter`
- OpenTelemetry SDK docs: `opentelemetry_sdk::metrics::exporter::PushMetricExporter`
- OpenTelemetry SDK docs: `opentelemetry_sdk::trace::SpanExporter`

---

### 3. Python Bindings for Exporter Types

**Question**: How should we expose exporter creation methods and types through Python bindings?

**Research Findings**:

- **Decision**: Add `metric_exporter()` and `span_exporter()` methods to `PyOtlpLibrary` that return Python-compatible exporter objects
- **Rationale**:
  - Follows existing Python bindings pattern (methods on `PyOtlpLibrary`)
  - Python objects can wrap Rust exporter types using PyO3
  - Python OpenTelemetry SDK integration is out of scope (tracked in Issue #6)

**Implementation Approach**:
- Add `#[pyclass]` struct `PyOtlpMetricExporter` wrapping `Arc<OtlpMetricExporter>`
- Add `#[pyclass]` struct `PyOtlpSpanExporter` wrapping `Arc<OtlpSpanExporter>`
- Add `metric_exporter(&self) -> PyResult<PyOtlpMetricExporter>` method to `PyOtlpLibrary`
- Add `span_exporter(&self) -> PyResult<PyOtlpSpanExporter>` method to `PyOtlpLibrary`
- Python objects expose methods that delegate to Rust exporters
- Note: Python OpenTelemetry SDK adapter classes are separate (Issue #6)

**Alternatives Considered**:
- **Direct Python OpenTelemetry SDK integration**: DEFERRED - Tracked in Issue #6, requires separate research
- **No Python bindings**: REJECTED - Breaks feature parity requirement between Rust and Python APIs

**References**:
- Existing Python bindings: `src/python/bindings.rs::PyOtlpLibrary`
- PyO3 documentation: https://pyo3.rs/

---

### 4. Error Conversion Strategy

**Question**: How should we convert `OtlpError` to `OTelSdkError` while preserving error context?

**Research Findings**:

- **Decision**: Convert to `OTelSdkError::InternalFailure` with error message containing original error details
- **Rationale**:
  - `OTelSdkError::InternalFailure` is appropriate for library-internal errors
  - Error message preserves context for debugging
  - Logging before conversion provides additional observability

**Implementation Approach**:
- Use `OTelSdkError::InternalFailure(format!("OtlpLibrary export failed: {}", e))`
- Log errors using `tracing::warn!` before conversion
- Include error type and message in converted error

**Alternatives Considered**:
- **Error mapping**: Map specific `OtlpError` variants to specific `OTelSdkError` types - REJECTED: Over-engineering, InternalFailure is appropriate
- **Error wrapping**: Wrap `OtlpError` in custom error type - REJECTED: Adds complexity without benefit

**References**:
- Existing pattern: `src/otlp/exporter.rs::FileMetricExporter::export`
- OpenTelemetry SDK: `opentelemetry_sdk::error::OTelSdkError`

---

### 5. Lifecycle Management

**Question**: How should exporters handle shutdown methods from OpenTelemetry SDK?

**Research Findings**:

- **Decision**: Exporters handle shutdown gracefully but do not shut down the library
- **Rationale**:
  - Library lifecycle is independent of exporter lifecycle
  - Multiple exporters may share the same library instance
  - Developers call `OtlpLibrary::shutdown()` separately when application shuts down

**Implementation Approach**:
- `PushMetricExporter::shutdown_with_timeout` - returns `Ok(())` immediately
- `SpanExporter::shutdown` - returns `Ok(())` immediately
- `PushMetricExporter::force_flush` - calls `library.flush()` if runtime handle is available
- Document that library shutdown is separate operation

**Alternatives Considered**:
- **Shutdown library on exporter shutdown**: REJECTED - Would break if multiple exporters share library
- **Track shutdown state**: REJECTED - Adds complexity, not needed since library handles its own state

**References**:
- Existing pattern: `src/otlp/exporter.rs::FileMetricExporter::shutdown_with_timeout`
- Spec requirement: FR-014, FR-023

---

### 6. Concurrent Use Support

**Question**: How do we ensure exporters support concurrent use from multiple OpenTelemetry SDK components?

**Research Findings**:

- **Decision**: Use `Arc<OtlpLibrary>` and ensure `OtlpLibrary` methods are thread-safe
- **Rationale**:
  - `OtlpLibrary` already uses `Arc` internally for shared state
  - `BatchBuffer` uses `tokio::sync::Mutex` for thread-safe access
  - Exporters are `Send + Sync` by design (wrapping `Arc`)

**Implementation Approach**:
- Exporters wrap `Arc<OtlpLibrary>` (already `Send + Sync`)
- Delegate to `OtlpLibrary` methods which are already thread-safe
- No additional synchronization needed

**Alternatives Considered**:
- **Additional mutexes**: REJECTED - Not needed, library is already thread-safe
- **Single-threaded only**: REJECTED - Would limit use cases unnecessarily

**References**:
- Existing code: `src/api/public.rs::OtlpLibrary` uses `Arc` internally
- Existing code: `src/otlp/batch_writer.rs::BatchBuffer` uses `tokio::sync::Mutex`

---

## Summary

All research questions resolved. Implementation will:

1. Add `export_metrics_ref(&ResourceMetrics)` method that converts reference to protobuf (same path as owned export)
2. Create `OtlpMetricExporter` and `OtlpSpanExporter` types that implement OpenTelemetry SDK traits
3. Add `metric_exporter()` and `span_exporter()` convenience methods to `OtlpLibrary`
4. Extend Python bindings with exporter creation methods and Python-compatible exporter types
5. Convert errors appropriately while preserving context
6. Handle lifecycle independently (exporters don't shut down library)
7. Support concurrent use through existing thread-safe `OtlpLibrary` design

No blocking technical issues identified. Implementation follows existing patterns and best practices.

