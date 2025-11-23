# Python API Contract: OTLP Arrow Flight Library

**Date**: 2024-11-23  
**Feature**: 001-otlp-arrow-library

## Overview

The library provides Python bindings (via PyO3) that mirror the Rust public API, allowing Python applications to use the same functionality as Rust applications. The Python API follows Python naming conventions (snake_case) and Pythonic error handling patterns.

## Python API Methods

### Library Initialization

#### `OtlpLibrary(config: Optional[Config] = None) -> OtlpLibrary`

Creates a new OTLP library instance with the provided configuration.

**Parameters**:
- `config: Optional[Config]` - Configuration object (defaults to Config.default() if None)

**Returns**: `OtlpLibrary` - Library instance

**Raises**:
- `ConfigError` - Invalid configuration values
- `IOError` - Failed to create output directory

**Example**:
```python
import otlp_arrow_library

# With default configuration
library = otlp_arrow_library.OtlpLibrary()

# With custom configuration
config = otlp_arrow_library.Config(
    output_dir="./custom_output",
    write_interval_secs=10
)
library = otlp_arrow_library.OtlpLibrary(config)
```

---

#### `OtlpLibrary.with_config_builder() -> ConfigBuilder`

Creates a configuration builder for programmatic configuration.

**Returns**: `ConfigBuilder` - Configuration builder

**Example**:
```python
library = otlp_arrow_library.OtlpLibrary.with_config_builder() \
    .output_dir("./custom_output") \
    .write_interval_secs(10) \
    .build()
```

---

### Trace Operations

#### `export_trace(span: SpanData) -> None`

Exports a single trace span to the library.

**Parameters**:
- `span: SpanData` - OpenTelemetry span data (from opentelemetry Python SDK)

**Raises**:
- `ExportError` - Failed to buffer or process span
- `IOError` - File system error

**Example**:
```python
from opentelemetry import trace

tracer = trace.get_tracer(__name__)
with tracer.start_as_current_span("my_span") as span:
    library.export_trace(span)
```

---

#### `export_traces(spans: List[SpanData]) -> None`

Exports multiple trace spans in a single call.

**Parameters**:
- `spans: List[SpanData]` - Collection of span data

**Raises**:
- `ExportError` - Failed to buffer or process spans
- `IOError` - File system error

**Example**:
```python
spans = [span1, span2, span3]
library.export_traces(spans)
```

---

### Metric Operations

#### `export_metrics(metrics: ResourceMetrics) -> None`

Exports metrics data to the library.

**Parameters**:
- `metrics: ResourceMetrics` - OpenTelemetry resource-scoped metrics

**Raises**:
- `ExportError` - Failed to buffer or process metrics
- `IOError` - File system error

**Example**:
```python
from opentelemetry import metrics

meter = metrics.get_meter(__name__)
counter = meter.create_counter("my_counter")
counter.add(10)
# Export metrics
library.export_metrics(metrics)
```

---

### Service Lifecycle

#### `flush() -> None`

Forces immediate flush of all buffered messages to disk.

**Raises**:
- `IOError` - File system error during flush

**Example**:
```python
library.flush()
```

---

#### `shutdown() -> None`

Gracefully shuts down the library, flushing all pending writes.

**Raises**:
- `ShutdownError` - Failed to flush or shutdown cleanly

**Example**:
```python
library.shutdown()
```

---

## Error Types

### `OtlpError`

Base exception class for all library errors.

**Subclasses**:
- `OtlpConfigError` - Configuration errors
- `OtlpExportError` - Export/processing errors
- `OtlpIOError` - I/O errors (extends IOError)
- `OtlpServerError` - Server errors

---

### `OtlpConfigError`

Configuration-related errors.

**Attributes**:
- `message: str` - Error message

**Example**:
```python
try:
    config = Config(output_dir="")
    library = OtlpLibrary(config)
except OtlpConfigError as e:
    print(f"Configuration error: {e.message}")
```

---

### `OtlpExportError`

Export/processing errors.

**Attributes**:
- `message: str` - Error message

**Example**:
```python
try:
    library.export_trace(span)
except OtlpExportError as e:
    print(f"Export failed: {e.message}")
```

---

## Thread Safety

All Python API methods are thread-safe and can be called from multiple threads concurrently. The underlying Rust implementation uses internal synchronization to ensure safe concurrent access.

---

## Async Support

The Python API supports both synchronous and asynchronous usage:

**Synchronous**:
```python
library.export_trace(span)  # Blocks until complete
```

**Asynchronous** (via asyncio):
```python
import asyncio

async def export_async():
    await library.export_trace_async(span)
```

---

## Configuration API

### `Config`

Main configuration class.

**Fields**:
- `output_dir: str` - Output directory path
- `write_interval_secs: int` - Write interval in seconds
- `trace_cleanup_interval_secs: int` - Trace cleanup interval
- `metric_cleanup_interval_secs: int` - Metric cleanup interval
- `forwarding: Optional[ForwardingConfig]` - Forwarding configuration

**Example**:
```python
config = otlp_arrow_library.Config(
    output_dir="./output",
    write_interval_secs=5,
    trace_cleanup_interval_secs=600,
    metric_cleanup_interval_secs=3600
)
```

### `ConfigBuilder`

Builder pattern for creating configurations.

**Methods**:
- `output_dir(path: str) -> ConfigBuilder`
- `write_interval_secs(secs: int) -> ConfigBuilder`
- `trace_cleanup_interval_secs(secs: int) -> ConfigBuilder`
- `metric_cleanup_interval_secs(secs: int) -> ConfigBuilder`
- `enable_forwarding(config: ForwardingConfig) -> ConfigBuilder`
- `build() -> Config`

---

## Mock Service API

### `MockOtlpService`

Mock service for testing.

**Methods**:
- `receive_trace(span: SpanData) -> None`
- `receive_metric(metrics: ResourceMetrics) -> None`
- `assert_traces_received(count: int) -> None` - Raises AssertionError if count doesn't match
- `assert_metrics_received(count: int) -> None` - Raises AssertionError if count doesn't match
- `reset() -> None`

**Example**:
```python
mock_service = otlp_arrow_library.MockOtlpService()
mock_service.receive_trace(span)
mock_service.assert_traces_received(1)
```

---

## Type Hints

The Python API includes type hints for better IDE support and type checking:

```python
from typing import List, Optional
from opentelemetry.trace import Span as SpanData
from opentelemetry.metrics import ResourceMetrics

def export_trace(span: SpanData) -> None: ...
def export_traces(spans: List[SpanData]) -> None: ...
def export_metrics(metrics: ResourceMetrics) -> None: ...
```

---

## Versioning

The Python API follows semantic versioning:
- **MAJOR**: Breaking changes to public API
- **MINOR**: New features, backward compatible
- **PATCH**: Bug fixes, backward compatible

Breaking changes will be documented in CHANGELOG.md with migration guides.

---

## Installation

The Python bindings are distributed as a Python wheel (`.whl`) and can be installed via pip:

```bash
pip install otlp-arrow-library
```

Or build from source:

```bash
pip install maturin
maturin develop
```

---

## Compatibility

- **Python**: 3.8+
- **Platforms**: Windows, Linux, macOS (same as Rust library)
- **OpenTelemetry Python SDK**: Compatible with opentelemetry-python SDK

