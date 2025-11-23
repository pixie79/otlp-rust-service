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

## Quick Start

### As a Standalone Service

```bash
# Run with default configuration
cargo run --bin otlp-arrow-service

# Run with custom config
OTLP_OUTPUT_DIR=./my_output cargo run --bin otlp-arrow-service
```

### As an Embedded Library (Rust)

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

    // Force flush
    library.flush().await?;

    // Shutdown gracefully
    library.shutdown().await?;
    
    Ok(())
}
```

### As an Embedded Library (Python)

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

# Flush and shutdown
library.flush()
library.shutdown()
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
