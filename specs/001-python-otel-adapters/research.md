# Research: Python OpenTelemetry SDK Adapter Classes

**Feature**: 001-python-otel-adapters  
**Date**: 2025-11-25  
**Status**: Complete

## Research Questions

### 1. Python OpenTelemetry SDK MetricExporter Interface

**Question**: What are the required methods and signatures for Python OpenTelemetry SDK's `MetricExporter` interface?

**Research Findings**:

- **Decision**: Implement Python OpenTelemetry SDK's `MetricExporter` abstract base class from `opentelemetry.sdk.metrics.export`
- **Rationale**: 
  - Python OpenTelemetry SDK defines `MetricExporter` as an abstract base class (ABC)
  - Required methods: `export(metrics_data)`, `shutdown()`, `force_flush()`
  - Optional methods: `temporality()` (returns Temporality enum)
  - Interface is stable in current Python OpenTelemetry SDK versions

**Implementation Approach**:
- Create Python class in Rust using PyO3 that implements the ABC
- Use PyO3's `#[pyclass]` with `#[pymethods]` to define Python methods
- Methods will delegate to underlying `OtlpLibrary` Python API
- Type conversion from Python OpenTelemetry SDK metric types to library-compatible formats

**Key Methods**:
- `export(metrics_data: MetricExportResult) -> ExportResult`: Export metrics data
- `shutdown() -> None`: Shutdown the exporter (no-op, library shutdown is separate)
- `force_flush(timeout_millis: Optional[int] = None) -> ExportResult`: Flush pending exports
- `temporality() -> Temporality`: Return temporality preference (default: CUMULATIVE)

**Alternatives Considered**:
- **Duck typing**: REJECTED - Python OpenTelemetry SDK uses ABC, requires proper inheritance
- **Wrapper in pure Python**: REJECTED - Would require maintaining separate Python code, loses type safety benefits of Rust

**References**:
- Python OpenTelemetry SDK: `opentelemetry.sdk.metrics.export.MetricExporter`
- PyO3 ABC support: PyO3 allows implementing Python ABCs from Rust

---

### 2. Python OpenTelemetry SDK SpanExporter Interface

**Question**: What are the required methods and signatures for Python OpenTelemetry SDK's `SpanExporter` interface?

**Research Findings**:

- **Decision**: Implement Python OpenTelemetry SDK's `SpanExporter` abstract base class from `opentelemetry.sdk.trace.export`
- **Rationale**:
  - Python OpenTelemetry SDK defines `SpanExporter` as an abstract base class (ABC)
  - Required methods: `export(spans)`, `shutdown()`, `force_flush()`
  - Interface is stable in current Python OpenTelemetry SDK versions

**Implementation Approach**:
- Create Python class in Rust using PyO3 that implements the ABC
- Use PyO3's `#[pyclass]` with `#[pymethods]` to define Python methods
- Methods will delegate to underlying `OtlpLibrary` Python API
- Type conversion from Python OpenTelemetry SDK span types to library-compatible formats

**Key Methods**:
- `export(spans: Sequence[ReadableSpan]) -> SpanExportResult`: Export span data
- `shutdown() -> None`: Shutdown the exporter (no-op, library shutdown is separate)
- `force_flush(timeout_millis: Optional[int] = None) -> SpanExportResult`: Flush pending exports

**Alternatives Considered**:
- **Duck typing**: REJECTED - Python OpenTelemetry SDK uses ABC, requires proper inheritance
- **Wrapper in pure Python**: REJECTED - Would require maintaining separate Python code, loses type safety benefits of Rust

**References**:
- Python OpenTelemetry SDK: `opentelemetry.sdk.trace.export.SpanExporter`
- PyO3 ABC support: PyO3 allows implementing Python ABCs from Rust

---

### 3. Type Conversion: Python OpenTelemetry SDK to Library Formats

**Question**: How do we convert Python OpenTelemetry SDK metric and span types to library-compatible formats?

**Research Findings**:

- **Decision**: Implement type conversion layer that converts Python OpenTelemetry SDK types to formats compatible with library's Python API
- **Rationale**:
  - Python OpenTelemetry SDK uses Python-specific types (e.g., `MetricExportResult`, `ReadableSpan`)
  - Library's Python API expects dictionary-based formats (as seen in existing `export_trace`, `export_metrics` methods)
  - Conversion must preserve 100% of data without loss or corruption

**Implementation Approach**:
- Extract data from Python OpenTelemetry SDK types using PyO3
- Convert to dictionary format compatible with library's existing Python API
- For metrics: Convert `MetricExportResult` → dictionary → `export_metrics_ref()` or `export_metrics()`
- For spans: Convert `ReadableSpan` sequence → list of dictionaries → `export_traces()`
- Handle nested structures (resource attributes, scope metrics, span attributes, events, links)

**Data Mapping**:
- **Metrics**: `MetricExportResult` contains `ResourceMetrics` with resource, scope metrics, metric data points
- **Spans**: `ReadableSpan` contains trace_id, span_id, name, kind, attributes, events, links, status, timestamps
- **Attributes**: Convert Python dict → Rust `KeyValue` → library format
- **Timestamps**: Convert Python datetime/nanoseconds → Rust `SystemTime`

**Alternatives Considered**:
- **Direct type usage**: REJECTED - Library API expects dictionary format, not Python OpenTelemetry SDK types
- **Protobuf conversion**: REJECTED - Would require additional conversion step, dictionary format is simpler
- **Extend library API**: REJECTED - Would break existing API, dictionary format is established pattern

**References**:
- Existing conversion: `src/python/bindings.rs::dict_to_span_data()` shows pattern for span conversion
- Existing API: `PyOtlpLibrary::export_trace()`, `export_traces()`, `export_metrics()` use dictionary format

---

### 4. PyO3 Interface Implementation Patterns

**Question**: How do we implement Python abstract base classes (ABC) from Rust using PyO3?

**Research Findings**:

- **Decision**: Use PyO3's `#[pyclass]` with Python-side registration to implement ABCs
- **Rationale**:
  - PyO3 supports implementing Python ABCs by registering the class and inheriting from the ABC in Python
  - Alternative: Use `PyClass` trait with `#[pyclass(subclass)]` for inheritance support
  - Best approach: Create Rust class, register in Python module, then create Python wrapper that inherits from ABC

**Implementation Approach**:
- Create Rust `#[pyclass]` structs for adapters (`PyOtlpMetricExporterAdapter`, `PyOtlpSpanExporterAdapter`)
- Implement `#[pymethods]` with required ABC methods
- In Python module initialization, create wrapper classes that inherit from Python OpenTelemetry SDK ABCs
- Wrapper classes delegate to Rust implementations
- Alternative simpler approach: Use duck typing if ABC inheritance proves complex

**Error Handling**:
- Convert Rust `PyResult<T>` to Python exceptions
- Preserve error context in exception messages
- Use appropriate Python exception types (`RuntimeError`, `ValueError`)

**Lifecycle Management**:
- Use `Py<PyOtlpLibrary>` to hold reference to library (prevents garbage collection)
- Ensure adapter objects remain valid while in use by Python OpenTelemetry SDK
- Handle Python garbage collection appropriately

**Alternatives Considered**:
- **Pure Rust implementation**: REJECTED - Cannot directly implement Python ABCs from Rust without Python-side registration
- **Pure Python wrapper**: REJECTED - Would duplicate logic, lose type safety benefits
- **Duck typing only**: ACCEPTABLE FALLBACK - If ABC inheritance proves too complex, duck typing may suffice

**References**:
- PyO3 documentation: https://pyo3.rs/latest/class.html
- PyO3 inheritance: https://pyo3.rs/latest/class/inheritance.html
- Existing pattern: `src/python/bindings.rs::PyOtlpLibrary` shows PyO3 class structure

---

### 5. Python OpenTelemetry SDK Integration Patterns

**Question**: How do `PeriodicExportingMetricReader` and `BatchSpanProcessor` use exporters?

**Research Findings**:

- **Decision**: Follow standard Python OpenTelemetry SDK integration patterns
- **Rationale**:
  - `PeriodicExportingMetricReader` takes a `MetricExporter` and calls `export()` periodically
  - `BatchSpanProcessor` takes a `SpanExporter` and calls `export()` when batches are ready
  - Both handle errors, retries, and shutdown automatically

**Integration Patterns**:
- **Metrics**: 
  ```python
  from opentelemetry.sdk.metrics.export import PeriodicExportingMetricReader
  reader = PeriodicExportingMetricReader(metric_exporter, export_interval_millis=5000)
  ```
- **Traces**:
  ```python
  from opentelemetry.sdk.trace.export import BatchSpanProcessor
  processor = BatchSpanProcessor(span_exporter)
  tracer_provider.add_span_processor(processor)
  ```

**Error Handling**:
- Python OpenTelemetry SDK handles retries and backoff automatically
- Exporters should return appropriate `ExportResult` types
- Errors are logged by Python OpenTelemetry SDK

**Lifecycle**:
- Python OpenTelemetry SDK manages exporter lifecycle
- Exporters receive `shutdown()` calls when SDK shuts down
- Adapters should handle shutdown gracefully (no-op, library shutdown is separate)

**Alternatives Considered**:
- **Custom reader/processor**: REJECTED - Standard patterns are sufficient, no need for custom implementations
- **Direct SDK integration**: REJECTED - Would require deeper SDK integration, standard patterns are preferred

**References**:
- Python OpenTelemetry SDK documentation: https://opentelemetry-python.readthedocs.io/
- Existing integration examples in Python OpenTelemetry SDK repository

---

### 6. Cross-Platform Compatibility

**Question**: What platform-specific considerations exist for Python bindings and type conversion?

**Research Findings**:

- **Decision**: Use PyO3's cross-platform abstractions, handle platform differences in type conversion
- **Rationale**:
  - PyO3 handles most platform differences automatically
  - Type conversion logic should be platform-agnostic
  - Testing on all platforms (Windows, Linux, macOS) will validate compatibility

**Platform Considerations**:
- **Windows**: Path handling, file system differences (handled by library, not adapters)
- **Linux**: Standard Unix behavior (no special handling needed)
- **macOS**: Similar to Linux (no special handling needed)
- **Python versions**: Support 3.11+ (as specified in requirements)

**Type Conversion**:
- Python types are consistent across platforms
- Timestamp handling may differ (nanoseconds vs system time) - use Python OpenTelemetry SDK's timestamp types
- Byte handling is consistent (Python bytes type is platform-agnostic)

**Testing Strategy**:
- Test on all three platforms in CI/CD
- Verify identical behavior across platforms
- Test with different Python versions (3.11, 3.12, 3.13)

**Alternatives Considered**:
- **Platform-specific implementations**: REJECTED - Not needed, PyO3 handles platform differences
- **Windows-only support**: REJECTED - Specification requires cross-platform support

**References**:
- PyO3 cross-platform support: https://pyo3.rs/latest/
- Existing cross-platform tests: `.github/workflows/` CI/CD workflows

---

## Summary

All research questions resolved. Implementation will:

1. Create Python adapter classes in Rust using PyO3 that implement Python OpenTelemetry SDK's `MetricExporter` and `SpanExporter` ABCs
2. Implement type conversion layer to convert Python OpenTelemetry SDK types to library-compatible dictionary formats
3. Delegate export operations to existing `OtlpLibrary` Python API methods
4. Handle errors, lifecycle, and garbage collection appropriately
5. Support standard Python OpenTelemetry SDK integration patterns (`PeriodicExportingMetricReader`, `BatchSpanProcessor`)
6. Ensure cross-platform compatibility (Windows, Linux, macOS) and Python 3.11+ support

**Key Implementation Notes**:
- ABC inheritance may require Python-side wrapper classes (simpler than pure Rust implementation)
- Type conversion follows existing patterns (`dict_to_span_data` shows the approach)
- Error handling preserves context while converting to Python exceptions
- Lifecycle management keeps adapters valid while in use by Python OpenTelemetry SDK

No blocking technical issues identified. Implementation follows existing PyO3 patterns and Python OpenTelemetry SDK conventions.

