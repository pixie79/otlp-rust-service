# OTLP Arrow Flight Library

A cross-platform Rust library for receiving OpenTelemetry Protocol (OTLP) messages via gRPC and writing them to local files in Arrow IPC Streaming format. The library supports both standalone service mode and embedded library usage with public API methods.

## Features

- **Dual Protocol Support**: Simultaneous support for gRPC Protobuf (standard OTLP) and gRPC Arrow Flight (OTAP) on different ports
- **Arrow IPC Storage**: Writes telemetry data to Arrow IPC Streaming format files with automatic rotation
- **Batch Writing**: Configurable write intervals for efficient disk I/O
- **File Cleanup**: Automatic cleanup of old trace and metric files based on configurable retention intervals
- **Public API**: Embedded library mode with programmatic API for Rust and Python
- **Configuration System**: Support for YAML files, environment variables, and programmatic API
- **Optional Forwarding**: Forward messages to remote OTLP endpoints with automatic format conversion
- **Format Conversion**: Automatic conversion between Protobuf and Arrow Flight formats
- **Authentication**: Support for API key, bearer token, and basic authentication in forwarding
- **Circuit Breaker**: Automatic failure handling with circuit breaker pattern for forwarding
- **Mock Service**: Built-in mock service for end-to-end testing
- **Python Bindings**: PyO3 bindings for Python integration
- **Health Check**: HTTP health check endpoint for standalone service
- **Metrics Collection**: Library operation metrics (messages received, files written, errors, conversions)
- **OpenTelemetry SDK Integration**: Built-in `PushMetricExporter` and `SpanExporter` implementations for seamless integration with OpenTelemetry SDK
- **Reference-Based Export**: Efficient metric export via `export_metrics_ref()` method that accepts references instead of requiring ownership

## Quick Start

### Demo Application (Recommended for First-Time Users)

The demo application is the easiest way to get started and verify the service is working:

```bash
# Run the demo application
cargo run --example demo-app
```

**Prerequisites**: For dashboard visualization, build the dashboard first:
```bash
cd dashboard && npm install && npm run build && cd ..
```

**If dashboard is not built**: The demo will still run and generate data, but without the web dashboard. Data will be written to `./output_dir/otlp/` and you can view it later after building the dashboard.

The demo will:
1. Start the dashboard on http://127.0.0.1:8080 (if dashboard is built)
2. Generate real metrics using OpenTelemetry SDK Meter API and mock spans continuously
3. Display data in real-time in the dashboard (if enabled)

**To view the dashboard**: 
1. Open your browser to http://127.0.0.1:8080
2. Click "Choose Directory" and select the `./output_dir/otlp` directory (the parent directory containing both `traces` and `metrics` subdirectories)
3. The dashboard will automatically load and display trace and metric data

**Important**: Select the `otlp` directory (not `otlp/traces` or `otlp/metrics`), as the dashboard needs access to both subdirectories.

**To stop the demo**: Press `Ctrl+C` (the demo will flush data and shutdown gracefully)

The demo application serves as both:
- **Verification tool**: Proves the service is working correctly
- **Reference implementation**: Shows how to use the SDK in your own applications

See `examples/demo-app.rs` for the complete source code with extensive comments.

### As a Standalone Service

```bash
# Run with default configuration
cargo run --bin otlp-arrow-service

# Run with custom config
OTLP_OUTPUT_DIR=./my_output cargo run --bin otlp-arrow-service
```

### As an Embedded Library (Rust)

#### Basic Usage

```rust
use otlp_arrow_library::{OtlpLibrary, Config};
use opentelemetry_sdk::trace::SpanData;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Create configuration
    let config = Config::default();
    
    // Initialize library
    let library = OtlpLibrary::new(config).await?;

    // Export a trace span
    // library.export_trace(span).await?;

    // Export multiple traces
    // library.export_traces(spans).await?;

    // Export metrics (automatically converted to protobuf for storage)
    // library.export_metrics(metrics).await?;

    // Export metrics by reference (more efficient)
    // library.export_metrics_ref(&metrics).await?;

    // Force flush
    library.flush().await?;

    // Shutdown gracefully
    library.shutdown().await?;
    
    Ok(())
}
```

#### Integration with OpenTelemetry SDK

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

    // Create exporters for OpenTelemetry SDK
    let metric_exporter = library.metric_exporter();
    let span_exporter = library.span_exporter();

    // Use with OpenTelemetry SDK
    let metric_reader = PeriodicReader::builder(metric_exporter)
        .with_interval(Duration::from_secs(10))
        .build();

    let meter_provider = MeterProvider::builder()
        .with_reader(metric_reader)
        .build();

    let tracer_provider = opentelemetry_sdk::trace::TracerProvider::builder()
        .with_batch_exporter(span_exporter, opentelemetry_sdk::runtime::Tokio)
        .build();

    // Use providers to create meters and tracers
    // Metrics and traces are automatically exported via exporters

    // Shutdown when done
    library.shutdown().await?;
    Ok(())
}
```

### As an Embedded Library (Python)

#### Basic Usage

```python
import otlp_arrow_library

# Initialize library
library = otlp_arrow_library.PyOtlpLibrary(
    output_dir="./output",
    write_interval_secs=5
)

# Export a trace
trace_id = bytes([1] * 16)
span_id = bytes([1] * 8)
span = {
    "trace_id": trace_id,
    "span_id": span_id,
    "name": "my-span",
    "kind": "server",
    "attributes": {"service.name": "my-service"}
}
library.export_trace(span)

# Export metrics by reference (more efficient)
metrics = {}  # Your metrics dictionary
library.export_metrics_ref(metrics)

# Flush and shutdown
library.flush()
library.shutdown()
```

#### Exporter Creation

```python
import otlp_arrow_library

# Initialize library
library = otlp_arrow_library.PyOtlpLibrary(
    output_dir="./output",
    write_interval_secs=5
)

# Create exporters for OpenTelemetry SDK integration
metric_exporter = library.metric_exporter()
span_exporter = library.span_exporter()

# Note: Direct Python OpenTelemetry SDK integration requires
# adapter classes (see Issue #6). For now, use library methods directly.
```

## Configuration

Configuration can be provided via:
- **YAML file**: `config.yaml`
- **Environment variables**: `OTLP_*` prefix (e.g., `OTLP_OUTPUT_DIR`, `OTLP_WRITE_INTERVAL_SECS`)
- **Programmatic API**: `ConfigBuilder`

### Configuration Options

- `output_dir`: Output directory for Arrow IPC files (default: `./output_dir`)
- `write_interval_secs`: How frequently to write batches to disk in seconds (default: 5)
- `trace_cleanup_interval_secs`: Trace file retention interval in seconds (default: 600)
- `metric_cleanup_interval_secs`: Metric file retention interval in seconds (default: 3600)
- `protocols`: Protocol configuration
  - `protobuf_enabled`: Enable gRPC Protobuf server (default: true)
  - `protobuf_port`: Port for Protobuf server (default: 4317)
  - `arrow_flight_enabled`: Enable gRPC Arrow Flight server (default: true)
  - `arrow_flight_port`: Port for Arrow Flight server (default: 4318)
- `forwarding`: Optional remote forwarding configuration
  - `enabled`: Enable forwarding (default: false)
  - `endpoint_url`: Remote endpoint URL (required if enabled)
  - `protocol`: Forwarding protocol (Protobuf or ArrowFlight, default: Protobuf)
  - `authentication`: Optional authentication configuration

### Example Configuration

#### YAML Configuration

```yaml
output_dir: ./custom_output
write_interval_secs: 10
trace_cleanup_interval_secs: 1200
metric_cleanup_interval_secs: 7200
protocols:
  protobuf_enabled: true
  protobuf_port: 4317
  arrow_flight_enabled: true
  arrow_flight_port: 4318
forwarding:
  enabled: true
  endpoint_url: "https://collector.example.com:4317"
  protocol: protobuf
  authentication:
    auth_type: bearer_token
    credentials:
      token: "my-bearer-token"
```

#### Environment Variables

```bash
export OTLP_OUTPUT_DIR=./my_output
export OTLP_WRITE_INTERVAL_SECS=10
export OTLP_PROTOBUF_ENABLED=true
export OTLP_ARROW_FLIGHT_ENABLED=true
```

#### Programmatic Configuration

```rust
use otlp_arrow_library::{ConfigBuilder, ForwardingConfig, ForwardingProtocol};

let config = ConfigBuilder::new()
    .output_dir("./custom_output")
    .write_interval_secs(10)
    .protobuf_enabled(true)
    .arrow_flight_enabled(true)
    .enable_forwarding(ForwardingConfig {
        enabled: true,
        endpoint_url: Some("https://collector.example.com:4317".to_string()),
        protocol: ForwardingProtocol::Protobuf,
        authentication: None,
    })
    .build()?;
```

See [quickstart guide](specs/001-otlp-arrow-library/quickstart.md) for detailed examples.

## Project Structure

```
src/
├── lib.rs          # Library root
├── config/         # Configuration management
├── otlp/           # OTLP processing (server, exporter, forwarder)
├── api/            # Public API
├── mock/           # Mock service for testing
└── bin/            # Standalone service binary

tests/
├── unit/           # Unit tests
├── integration/    # Integration tests
└── contract/       # Contract tests
```

## Requirements

- Rust 1.75+ (stable channel)
- Tokio async runtime
- Cross-platform: Windows, Linux, macOS
- Python 3.11+ (for Python bindings)

## CI/CD

The project includes GitHub Actions workflows for:
- **Cross-platform testing**: Runs tests on Windows, Linux, and macOS
- **Code coverage validation**: Ensures 85% coverage per file requirement
- **Linting and formatting**: Checks code style and formatting
- **Build verification**: Validates builds on all platforms
- **Python bindings**: Builds and tests Python wheels

See `.github/workflows/` for workflow definitions.

## Building

### Rust Library

```bash
# Build library
cargo build --release

# Run tests
cargo test

# Run benchmarks
cargo bench
```

### Python Bindings

```bash
# Install maturin
pip install maturin

# Build Python wheel
maturin build --release

# Or install in development mode
maturin develop
```

## API Documentation

### Rust API

See [API documentation](src/api/public.rs) for complete Rust API reference.

### Python API

See [Python API contract](specs/001-otlp-arrow-library/contracts/python-api.md) for Python API reference.

## Examples

### Demo Application

The demo application (`examples/demo-app.rs`) is the recommended starting point for new users:

**Run the demo**:
```bash
cargo run --example demo-app
```

**What it does**:
- Enables the dashboard automatically (http://127.0.0.1:8080)
- Generates mock metrics and spans continuously
- Demonstrates all SDK usage patterns with extensive comments
- Shows parent-child span relationships and different span kinds
- Includes graceful shutdown on Ctrl+C

**Use cases**:
- Verify the service is working after installation
- See a complete reference implementation
- Learn SDK usage patterns by reading the well-commented code

### Other Examples

- **Standalone Service**: `examples/standalone.rs` - Run as a standalone gRPC service
- **Embedded Library**: `examples/embedded.rs` - Use as an embedded component
- **Python Usage**: `examples/python_example.py` - Python integration example

## Health Check

The standalone service includes a health check endpoint on port 8080:

```bash
curl http://localhost:8080
# Returns: OK
```

## Metrics

The library collects operation metrics that can be accessed via `OtlpFileExporter::get_metrics()`:

- Messages received (traces + metrics)
- Files written
- Errors encountered
- Format conversions performed

## Implementation Notes

### ResourceMetrics Storage

The library uses protobuf format (`ExportMetricsServiceRequest`) for internal metric storage to solve the `ResourceMetrics` Clone limitation. When you call `export_metrics()` with a `ResourceMetrics` instance:

1. The metrics are automatically converted to protobuf format for storage
2. Protobuf format supports `Clone`, enabling proper buffering and forwarding
3. When flushing, protobuf is converted back to `ResourceMetrics` for file export
4. This ensures full data preservation and proper handling of metrics from all sources (gRPC Protobuf, Arrow Flight, and public API)

## Documentation

- [Specification](specs/001-otlp-arrow-library/spec.md)
- [Implementation Plan](specs/001-otlp-arrow-library/plan.md)
- [Quickstart Guide](specs/001-otlp-arrow-library/quickstart.md)
- [API Contracts](specs/001-otlp-arrow-library/contracts/)

## License

MIT OR Apache-2.0
