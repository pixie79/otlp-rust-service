# Python API Contract: Built-in OpenTelemetry Exporter Implementations

**Date**: 2025-01-27  
**Feature**: 003-opentelemetry-exporters

## Overview

This contract defines the Python API extensions for built-in OpenTelemetry SDK exporter implementations. These extensions provide Python developers with the same convenience methods available in Rust, enabling feature parity between Rust and Python APIs.

**Note**: Python OpenTelemetry SDK adapter classes (that implement Python OpenTelemetry SDK interfaces) are tracked separately in [Issue #6](https://github.com/pixie79/otlp-rust-service/issues/6). This contract covers the Python bindings for the Rust exporter types.

## Extended API Methods

### PyOtlpLibrary Extensions

#### `PyOtlpLibrary.metric_exporter() -> PyOtlpMetricExporter`

Creates a Python-compatible metric exporter object that wraps the Rust `OtlpMetricExporter`.

**Returns**: `PyOtlpMetricExporter` - Python-compatible metric exporter

**Raises**:
- `RuntimeError` - If library instance is invalid or exporter creation fails

**Example**:
```python
import otlp_arrow_library

library = otlp_arrow_library.OtlpLibrary()
metric_exporter = library.metric_exporter()
# Exporter can be used with Python code, but Python OpenTelemetry SDK
# integration requires adapter classes (see Issue #6)
```

---

#### `PyOtlpLibrary.span_exporter() -> PyOtlpSpanExporter`

Creates a Python-compatible span exporter object that wraps the Rust `OtlpSpanExporter`.

**Returns**: `PyOtlpSpanExporter` - Python-compatible span exporter

**Raises**:
- `RuntimeError` - If library instance is invalid or exporter creation fails

**Example**:
```python
import otlp_arrow_library

library = otlp_arrow_library.OtlpLibrary()
span_exporter = library.span_exporter()
# Exporter can be used with Python code, but Python OpenTelemetry SDK
# integration requires adapter classes (see Issue #6)
```

---

## New Types

### PyOtlpMetricExporter

**Type**: Python class wrapping `OtlpMetricExporter`

**Purpose**: Python-compatible interface to Rust metric exporter

**Methods**: TBD (implementation detail - may expose internal methods or be used internally)

**Usage**:
```python
library = otlp_arrow_library.OtlpLibrary()
exporter = library.metric_exporter()
# Exporter object available for use
```

**Note**: Direct integration with Python OpenTelemetry SDK requires adapter classes (see Issue #6). This type provides the foundation for such adapters.

---

### PyOtlpSpanExporter

**Type**: Python class wrapping `OtlpSpanExporter`

**Purpose**: Python-compatible interface to Rust span exporter

**Methods**: TBD (implementation detail - may expose internal methods or be used internally)

**Usage**:
```python
library = otlp_arrow_library.OtlpLibrary()
exporter = library.span_exporter()
# Exporter object available for use
```

**Note**: Direct integration with Python OpenTelemetry SDK requires adapter classes (see Issue #6). This type provides the foundation for such adapters.

---

## Python OpenTelemetry SDK Integration

### Current State

Python bindings expose the exporter creation methods and types, but do not directly implement Python OpenTelemetry SDK exporter interfaces.

### Future Enhancement

Python adapter classes that implement Python OpenTelemetry SDK interfaces (`MetricExporter`, `SpanExporter`) are tracked in [Issue #6](https://github.com/pixie79/otlp-rust-service/issues/6).

### Workaround

Python developers can:
1. Use `PyOtlpLibrary` methods directly (`export_metrics`, `export_traces`)
2. Create custom adapter classes that implement Python OpenTelemetry SDK interfaces
3. Call library methods from adapter implementations

---

## Error Handling

### Error Types

- `RuntimeError` - Raised for all errors (library errors, export failures, etc.)
- Error messages include context from underlying Rust errors

**Example**:
```python
try:
    exporter = library.metric_exporter()
except RuntimeError as e:
    print(f"Failed to create exporter: {e}")
```

---

## Lifecycle Management

### Exporter Lifecycle

- **Creation**: Exporters created via `PyOtlpLibrary` convenience methods
- **Usage**: Exporters can be used in Python code
- **Shutdown**: Exporters handle shutdown gracefully but do not shut down library

### Library Lifecycle

- **Independent**: Library lifecycle is independent of exporter lifecycle
- **Multiple Exporters**: Multiple exporters can share the same library instance
- **Shutdown**: Developers call `PyOtlpLibrary.shutdown()` separately when application shuts down

---

## Thread Safety

- Exporters are thread-safe (wrap thread-safe Rust types)
- Can be used from multiple Python threads
- Python GIL handling managed by PyO3

---

## Platform Support

- **Python Versions**: Python 3.11 or higher
- **Platforms**: Windows, Linux, macOS
- **Architecture**: Same as Rust library (x86_64, ARM64, etc.)

---

## Backward Compatibility

- All existing `PyOtlpLibrary` methods remain unchanged
- New methods are additive (no breaking changes)
- Existing Python code continues to work without modification

---

## Usage Examples

### Basic Usage

```python
import otlp_arrow_library

# Create library instance
library = otlp_arrow_library.OtlpLibrary()

# Create exporters
metric_exporter = library.metric_exporter()
span_exporter = library.span_exporter()

# Exporters are available for use
# (Python OpenTelemetry SDK integration requires Issue #6)
```

### With Custom Configuration

```python
import otlp_arrow_library

# Create library with custom config
library = otlp_arrow_library.OtlpLibrary(
    output_dir="./custom_output",
    write_interval_secs=10
)

# Create exporters (use library's configuration)
metric_exporter = library.metric_exporter()
span_exporter = library.span_exporter()
```

---

## Future Enhancements

See [Issue #6](https://github.com/pixie79/otlp-rust-service/issues/6) for Python OpenTelemetry SDK adapter classes that would enable direct integration:

```python
# Future (after Issue #6):
from opentelemetry.sdk.metrics.export import PeriodicExportingMetricReader

library = otlp_arrow_library.OtlpLibrary()
metric_exporter = library.metric_exporter()  # Returns Python OpenTelemetry SDK MetricExporter
reader = PeriodicExportingMetricReader(metric_exporter)
```

