# Quickstart Guide: Python OpenTelemetry SDK Adapter Classes

**Feature**: 001-python-otel-adapters  
**Date**: 2025-11-25

## Overview

This guide demonstrates how to use Python OpenTelemetry SDK adapter classes to integrate `OtlpLibrary` with Python OpenTelemetry SDK's metric and trace collection systems.

## Prerequisites

- Python 3.11 or higher
- `otlp-arrow-library` Python package installed
- Python OpenTelemetry SDK packages installed:
  - `opentelemetry-api`
  - `opentelemetry-sdk`
  - `opentelemetry-instrumentation` (optional, for auto-instrumentation)

## Installation

### Install Python Package

```bash
# Install from wheel (after building)
pip install otlp-arrow-library

# Or install in development mode
maturin develop
```

### Install Python OpenTelemetry SDK

```bash
pip install opentelemetry-api opentelemetry-sdk
```

## Basic Usage

### Metrics Integration

#### Step 1: Create Library and Adapter

```python
import otlp_arrow_library
from opentelemetry.sdk.metrics.export import PeriodicExportingMetricReader

# Create library instance
library = otlp_arrow_library.PyOtlpLibrary(
    output_dir="./otlp_output",
    write_interval_secs=5
)

# Create metric exporter adapter
metric_exporter = library.metric_exporter_adapter()
```

#### Step 2: Configure Python OpenTelemetry SDK

```python
from opentelemetry.sdk.metrics import MeterProvider
from opentelemetry.sdk.metrics.export import PeriodicExportingMetricReader

# Create reader with adapter
reader = PeriodicExportingMetricReader(
    metric_exporter,
    export_interval_millis=5000
)

# Create meter provider
meter_provider = MeterProvider(metric_readers=[reader])

# Set global meter provider
from opentelemetry import metrics
metrics.set_meter_provider(meter_provider)
```

#### Step 3: Use Metrics

```python
from opentelemetry import metrics

# Get meter
meter = metrics.get_meter(__name__)

# Create counter
counter = meter.create_counter(
    "requests_total",
    description="Total number of requests"
)

# Record metric
counter.add(1, {"method": "GET", "status": "200"})
```

#### Complete Example

```python
import otlp_arrow_library
from opentelemetry import metrics
from opentelemetry.sdk.metrics import MeterProvider
from opentelemetry.sdk.metrics.export import PeriodicExportingMetricReader

# Setup
library = otlp_arrow_library.PyOtlpLibrary(output_dir="./otlp_output")
metric_exporter = library.metric_exporter()
reader = PeriodicExportingMetricReader(metric_exporter, export_interval_millis=5000)
meter_provider = MeterProvider(metric_readers=[reader])
metrics.set_meter_provider(meter_provider)

# Use metrics
meter = metrics.get_meter(__name__)
counter = meter.create_counter("requests_total")
counter.add(1)

# Cleanup (when application shuts down)
library.shutdown()
```

---

### Traces Integration

#### Step 1: Create Library and Adapter

```python
import otlp_arrow_library
from opentelemetry.sdk.trace.export import BatchSpanProcessor

# Create library instance
library = otlp_arrow_library.PyOtlpLibrary(
    output_dir="./otlp_output",
    write_interval_secs=5
)

# Create span exporter adapter
span_exporter = library.span_exporter_adapter()
```

#### Step 2: Configure Python OpenTelemetry SDK

```python
from opentelemetry.sdk.trace import TracerProvider
from opentelemetry.sdk.trace.export import BatchSpanProcessor

# Create span processor with adapter
processor = BatchSpanProcessor(span_exporter)

# Create tracer provider
tracer_provider = TracerProvider()
tracer_provider.add_span_processor(processor)

# Set global tracer provider
from opentelemetry import trace
trace.set_tracer_provider(tracer_provider)
```

#### Step 3: Use Traces

```python
from opentelemetry import trace

# Get tracer
tracer = trace.get_tracer(__name__)

# Create span
with tracer.start_as_current_span("my_operation") as span:
    span.set_attribute("key", "value")
    # Your code here
```

#### Complete Example

```python
import otlp_arrow_library
from opentelemetry import trace
from opentelemetry.sdk.trace import TracerProvider
from opentelemetry.sdk.trace.export import BatchSpanProcessor

# Setup
library = otlp_arrow_library.PyOtlpLibrary(output_dir="./otlp_output")
span_exporter = library.span_exporter()
processor = BatchSpanProcessor(span_exporter)
tracer_provider = TracerProvider()
tracer_provider.add_span_processor(processor)
trace.set_tracer_provider(tracer_provider)

# Use traces
tracer = trace.get_tracer(__name__)
with tracer.start_as_current_span("my_operation") as span:
    span.set_attribute("key", "value")

# Cleanup (when application shuts down)
library.shutdown()
```

---

### Combined Metrics and Traces

```python
import otlp_arrow_library
from opentelemetry import metrics, trace
from opentelemetry.sdk.metrics import MeterProvider
from opentelemetry.sdk.metrics.export import PeriodicExportingMetricReader
from opentelemetry.sdk.trace import TracerProvider
from opentelemetry.sdk.trace.export import BatchSpanProcessor

# Create library instance
library = otlp_arrow_library.PyOtlpLibrary(output_dir="./otlp_output")

# Setup metrics
metric_exporter = library.metric_exporter()
metric_reader = PeriodicExportingMetricReader(metric_exporter, export_interval_millis=5000)
meter_provider = MeterProvider(metric_readers=[metric_reader])
metrics.set_meter_provider(meter_provider)

# Setup traces
span_exporter = library.span_exporter()
span_processor = BatchSpanProcessor(span_exporter)
tracer_provider = TracerProvider()
tracer_provider.add_span_processor(span_processor)
trace.set_tracer_provider(tracer_provider)

# Use both
meter = metrics.get_meter(__name__)
counter = meter.create_counter("requests_total")
counter.add(1)

tracer = trace.get_tracer(__name__)
with tracer.start_as_current_span("my_operation") as span:
    span.set_attribute("key", "value")

# Cleanup
library.shutdown()
```

## Configuration

### Library Configuration

```python
library = otlp_arrow_library.PyOtlpLibrary(
    output_dir="./otlp_output",           # Output directory
    write_interval_secs=5,                 # Write interval
    trace_cleanup_interval_secs=600,      # Trace cleanup interval
    metric_cleanup_interval_secs=3600,    # Metric cleanup interval
    protobuf_enabled=True,                # Enable Protobuf protocol
    protobuf_port=4317,                   # Protobuf port
    arrow_flight_enabled=True,            # Enable Arrow Flight protocol
    arrow_flight_port=4318                # Arrow Flight port
)
```

### Python OpenTelemetry SDK Configuration

#### Metrics Reader Configuration

```python
reader = PeriodicExportingMetricReader(
    metric_exporter,
    export_interval_millis=5000,  # Export interval
    timeout_millis=30000          # Export timeout
)
```

#### Span Processor Configuration

```python
processor = BatchSpanProcessor(
    span_exporter,
    max_queue_size=2048,          # Maximum queue size
    export_timeout_millis=30000,   # Export timeout
    schedule_delay_millis=5000     # Schedule delay
)
```

## Common Patterns

### Custom Resource Attributes

```python
from opentelemetry.sdk.resources import Resource

resource = Resource.create({
    "service.name": "my-service",
    "service.version": "1.0.0"
})

meter_provider = MeterProvider(
    metric_readers=[reader],
    resource=resource
)

tracer_provider = TracerProvider(resource=resource)
```

### Error Handling

```python
try:
    metric_exporter = library.metric_exporter()
except RuntimeError as e:
    print(f"Failed to create metric exporter: {e}")
    # Handle error
```

### Flush Before Shutdown

```python
# Flush pending exports
library.flush()

# Shutdown library
library.shutdown()
```

## Troubleshooting

### Adapter Creation Fails

**Problem**: `RuntimeError` when creating adapter

**Solutions**:
- Verify library instance is valid (not shut down)
- Check library configuration is correct
- Verify Python OpenTelemetry SDK is installed

### Metrics/Spans Not Exported

**Problem**: Metrics or spans not appearing in output

**Solutions**:
- Check export intervals are configured correctly
- Verify library output directory is writable
- Check for errors in Python OpenTelemetry SDK logs
- Call `library.flush()` to force immediate export

### Type Conversion Errors

**Problem**: `RuntimeError` during type conversion

**Solutions**:
- Verify Python OpenTelemetry SDK version compatibility
- Check metric/span data structure is valid
- Review error messages for specific conversion issues

## Best Practices

1. **Single Library Instance**: Create one library instance and reuse adapters
2. **Proper Shutdown**: Always call `library.shutdown()` when application exits
3. **Error Handling**: Wrap adapter creation and usage in try-except blocks
4. **Resource Management**: Use context managers or ensure cleanup in finally blocks
5. **Configuration**: Use environment variables for configuration when possible
6. **Testing**: Test adapters with Python OpenTelemetry SDK test utilities

## Next Steps

- See [API Contract](contracts/python-api.md) for detailed API documentation
- See [Type Conversion Contract](contracts/type-conversion.md) for type conversion details
- See [Data Model](data-model.md) for entity definitions
- See [Specification](spec.md) for complete feature specification

