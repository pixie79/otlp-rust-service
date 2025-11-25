# Quickstart: Built-in OpenTelemetry Exporter Implementations

**Feature**: 003-opentelemetry-exporters  
**Date**: 2025-01-27

## Overview

This quickstart guide demonstrates how to use the built-in OpenTelemetry SDK exporter implementations to integrate `OtlpLibrary` with OpenTelemetry SDK without writing custom wrapper code.

## Prerequisites

- Rust project with `otlp-arrow-library` dependency
- OpenTelemetry SDK 0.31 dependency
- Tokio async runtime

## Rust Quickstart

### 1. Add Dependencies

```toml
[dependencies]
otlp-arrow-library = { path = "../otlp-rust-service" }
opentelemetry = "0.31"
opentelemetry-sdk = { version = "0.31", features = ["rt-tokio", "metrics", "trace"] }
tokio = { version = "1.35", features = ["full"] }
```

### 2. Create Library and Exporters

```rust
use otlp_arrow_library::OtlpLibrary;
use opentelemetry_sdk::metrics::{MeterProvider, PeriodicReader};
use opentelemetry_sdk::trace::TracerProvider;
use std::time::Duration;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Create library instance
    let config = otlp_arrow_library::Config::default();
    let library = OtlpLibrary::new(config).await?;

    // Create exporters
    let metric_exporter = library.metric_exporter();
    let span_exporter = library.span_exporter();

    // Use with OpenTelemetry SDK
    let metric_reader = PeriodicReader::builder(metric_exporter)
        .with_interval(Duration::from_secs(10))
        .build();

    let meter_provider = MeterProvider::builder()
        .with_reader(metric_reader)
        .build();

    let tracer_provider = TracerProvider::builder()
        .with_batch_exporter(span_exporter, opentelemetry_sdk::runtime::Tokio)
        .build();

    // Use providers to create meters and tracers
    // Metrics and traces are automatically exported via exporters

    // Shutdown when done
    library.shutdown().await?;
    Ok(())
}
```

### 3. Using Metrics

```rust
use opentelemetry::metrics::MeterProvider as _;

let meter = meter_provider.meter("my_app");
let counter = meter.u64_counter("requests").init();

// Record metrics - automatically exported via PeriodicReader
counter.add(1, &[]);
```

### 4. Using Traces

```rust
use opentelemetry::trace::TracerProvider as _;

let tracer = tracer_provider.tracer("my_app");
let mut span = tracer.start("my_operation");

// Do work
span.set_attribute(opentelemetry::KeyValue::new("key", "value"));

// Span automatically exported when finished
span.end();
```

## Python Quickstart

### 1. Install Library

```bash
# Build and install Python bindings
cd otlp-rust-service
maturin develop  # or maturin build for wheel
```

### 2. Create Library and Exporters

```python
import otlp_arrow_library

# Create library instance
library = otlp_arrow_library.OtlpLibrary()

# Create exporters
metric_exporter = library.metric_exporter()
span_exporter = library.span_exporter()

# Note: Direct Python OpenTelemetry SDK integration requires
# adapter classes (see Issue #6). For now, use library methods directly:
library.export_metrics(metrics)
library.export_traces(spans)
```

### 3. Future: Python OpenTelemetry SDK Integration

Once [Issue #6](https://github.com/pixie79/otlp-rust-service/issues/6) is implemented:

```python
from opentelemetry.sdk.metrics.export import PeriodicExportingMetricReader
from opentelemetry.sdk.trace.export import BatchSpanProcessor

library = otlp_arrow_library.OtlpLibrary()
metric_exporter = library.metric_exporter()  # Returns Python OpenTelemetry SDK MetricExporter
span_exporter = library.span_exporter()      # Returns Python OpenTelemetry SDK SpanExporter

reader = PeriodicExportingMetricReader(metric_exporter)
processor = BatchSpanProcessor(span_exporter)

# Use with Python OpenTelemetry SDK
```

## Reference-Based Export (Advanced)

For direct integration without exporters:

```rust
use otlp_arrow_library::OtlpLibrary;
use opentelemetry_sdk::metrics::data::ResourceMetrics;

let library = OtlpLibrary::new(config).await?;
let metrics: ResourceMetrics = /* ... */;

// Export by reference (more efficient than owned export)
library.export_metrics_ref(&metrics).await?;
```

## Configuration

Exporters use the same configuration as the library:

```rust
use otlp_arrow_library::OtlpLibrary;

let config = OtlpLibrary::with_config_builder()
    .output_dir("./custom_output")
    .write_interval_secs(10)
    .build()?;

let library = OtlpLibrary::new(config).await?;
let exporter = library.metric_exporter();
// Exporter uses library's configuration
```

## Error Handling

Exporters convert library errors to OpenTelemetry SDK errors:

```rust
// Errors are automatically converted
// Library errors -> OTelSdkError::InternalFailure
// Errors are logged before conversion
```

## Lifecycle Management

```rust
// Exporters handle their own shutdown
// Library shutdown is separate
library.shutdown().await?;
```

## Complete Example

See `examples/embedded.rs` for a complete working example.

## Next Steps

- Read [Rust API Contract](./contracts/rust-api.md) for detailed API documentation
- Read [Python API Contract](./contracts/python-api.md) for Python-specific details
- See [Issue #6](https://github.com/pixie79/otlp-rust-service/issues/6) for Python OpenTelemetry SDK adapter classes

