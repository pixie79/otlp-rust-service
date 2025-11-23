# Quickstart Guide: OTLP Arrow Flight Library

**Date**: 2024-11-23  
**Feature**: 001-otlp-arrow-library

## Overview

This guide provides a quick introduction to using the OTLP Arrow Flight Library. The library can be used in two modes:
1. **Standalone Service**: Run as a service that receives OTLP messages via gRPC
2. **Embedded Library**: Use public API methods to send OTLP messages from your application

## Installation

### Rust

Add to your `Cargo.toml`:

```toml
[dependencies]
otlp-arrow-library = { version = "0.1.0", path = "../otlp-arrow-library" }
opentelemetry = "0.31"
opentelemetry-sdk = "0.31"
```

### Python

Install from wheel or build from source:

```bash
# From wheel (when published)
pip install otlp-arrow-library

# From source
pip install maturin
maturin develop
```

## Standalone Service Mode

### Basic Usage

1. **Create configuration file** (`config.yaml`):
```yaml
output_dir: ./output_dir
write_interval_secs: 5
trace_cleanup_interval_secs: 600
metric_cleanup_interval_secs: 3600
```

2. **Run the service**:
```bash
otlp-arrow-library --config config.yaml
```

3. **Send OTLP messages** via gRPC:
   - Protobuf (standard OTLP): `localhost:4317`
   - Arrow Flight (OTAP): `localhost:4318`

### Configuration Options

**Environment Variables**:
```bash
export OTLP_OUTPUT_DIR=./custom_output
export OTLP_WRITE_INTERVAL_SECS=10
export OTLP_TRACE_CLEANUP_INTERVAL_SECS=300
export OTLP_METRIC_CLEANUP_INTERVAL_SECS=1800
export OTLP_PROTOBUF_ENABLED=true
export OTLP_PROTOBUF_PORT=4317
export OTLP_ARROW_FLIGHT_ENABLED=true
export OTLP_ARROW_FLIGHT_PORT=4318
```

**YAML Configuration**:
```yaml
output_dir: ./output_dir
write_interval_secs: 5
trace_cleanup_interval_secs: 600
metric_cleanup_interval_secs: 3600

protocols:
  protobuf_enabled: true
  protobuf_port: 4317
  arrow_flight_enabled: true
  arrow_flight_port: 4318

forwarding:
  enabled: true
  endpoint_url: "https://collector.example.com:4317"
  protocol: "protobuf"  # or "arrow_flight" - messages will be converted if received in different format
  authentication:
    auth_type: "api_key"
    credentials:
      api_key: "your-api-key"
```

### Output Files

Files are written to:
- Traces: `{OUTPUT_DIR}/otlp/traces/otlp_traces_YYYYMMDD_HHMMSS_NNNN.arrow`
- Metrics: `{OUTPUT_DIR}/otlp/metrics/otlp_metrics_YYYYMMDD_HHMMSS_NNNN.arrow`

Files are in Arrow IPC Streaming format and can be read by standard Arrow libraries.

---

## Embedded Library Mode

### Rust Usage

```rust
use otlp_arrow_library::{OtlpLibrary, Config};
use opentelemetry_sdk::trace::SpanData;

// Create library with default configuration
let config = Config::default();
let library = OtlpLibrary::new(config)?;

// Export a trace
let span: SpanData = /* create your span */;
library.export_trace(span)?;

// Export metrics
use opentelemetry_sdk::metrics::data::ResourceMetrics;
let metrics: ResourceMetrics = /* create your metrics */;
library.export_metrics(metrics)?;

// Flush and shutdown
library.flush()?;
library.shutdown()?;
```

### Python Usage

```python
import otlp_arrow_library

# Create library with default configuration
library = otlp_arrow_library.OtlpLibrary()

# Export a trace
span = # create your span (OpenTelemetry Python SDK)
library.export_trace(span)

# Export metrics
metrics = # create your metrics (OpenTelemetry Python SDK)
library.export_metrics(metrics)

# Flush and shutdown
library.flush()
library.shutdown()
```

### Custom Configuration

```rust
use otlp_arrow_library::{OtlpLibrary, ConfigBuilder};

let library = OtlpLibrary::with_config_builder()
    .output_dir("./my_output")
    .write_interval_secs(10)
    .trace_cleanup_interval_secs(300)
    .metric_cleanup_interval_secs(1800)
    .build()?;
```

### Async Usage

```rust
use otlp_arrow_library::OtlpLibrary;

let library = OtlpLibrary::new(Config::default())?;

// Async export
library.export_trace_async(span).await?;
library.export_metrics_async(metrics).await?;
```

---

## Remote Forwarding (Optional)

The library supports forwarding OTLP messages to remote endpoints with automatic format conversion. You can configure the forwarding protocol (Protobuf or Arrow Flight) independently of the input protocol. If messages are received in a different format than the configured forwarding format, they will be automatically converted.

### Format Conversion

The library automatically converts messages between Protobuf and Arrow Flight formats when forwarding:
- **Protobuf → Arrow Flight**: Messages received via Protobuf gRPC are converted to Arrow Flight format when forwarding to an Arrow Flight endpoint
- **Arrow Flight → Protobuf**: Messages received via Arrow Flight gRPC are converted to Protobuf format when forwarding to a Protobuf endpoint
- **Same format**: No conversion needed when input and forwarding formats match

Format conversion preserves all message data, attributes, and metadata. Conversion errors are logged and handled gracefully without blocking local storage.

### Enable Forwarding

**Via Configuration**:
```yaml
forwarding:
  enabled: true
  endpoint_url: "https://collector.example.com:4317"
  authentication:
    auth_type: "bearer_token"
    credentials:
      token: "your-token"
```

**Via Programmatic API**:
```rust
use otlp_arrow_library::{ForwardingConfig, ForwardingProtocol, AuthConfig};
use std::collections::HashMap;

let mut creds = HashMap::new();
creds.insert("token".to_string(), "your-token".to_string());

let forwarding = ForwardingConfig {
    enabled: true,
    endpoint_url: "https://collector.example.com:4317".to_string(),
    protocol: ForwardingProtocol::Protobuf, // or ForwardingProtocol::ArrowFlight
    authentication: Some(AuthConfig {
        auth_type: "bearer_token".to_string(),
        credentials: creds,
    }),
};

let config = ConfigBuilder::default()
    .enable_forwarding(forwarding)
    .build()?;
```

---

## Testing with Mock Service

### Using Mock Service in Tests

```rust
use otlp_arrow_library::mock::MockOtlpService;

#[tokio::test]
async fn test_otlp_export() {
    let mock_service = MockOtlpService::new();
    let addr = mock_service.start().await?;
    
    // Create library pointing to mock service
    let config = ConfigBuilder::default()
        .forwarding(ForwardingConfig {
            enabled: true,
            endpoint_url: format!("http://{}", addr),
            protocol: ForwardingProtocol::Protobuf, // or ForwardingProtocol::ArrowFlight
            authentication: None,
        })
        .build()?;
    
    let library = OtlpLibrary::new(config)?;
    
    // Export traces
    library.export_trace(span)?;
    
    // Assert mock service received the trace
    mock_service.assert_traces_received(1).await?;
}
```

### Testing gRPC Interface

```rust
use opentelemetry_proto::tonic::collector::trace::v1::trace_service_client::TraceServiceClient;
use opentelemetry_proto::tonic::collector::trace::v1::ExportTraceServiceRequest;

#[tokio::test]
async fn test_grpc_interface() {
    let mock_service = MockOtlpService::new();
    let addr = mock_service.start().await?;
    
    // Connect to mock service
    let mut client = TraceServiceClient::connect(format!("http://{}", addr)).await?;
    
    // Send trace request
    let request = ExportTraceServiceRequest { /* ... */ };
    let response = client.export(request).await?;
    
    // Assert received
    mock_service.assert_traces_received(1).await?;
}
```

---

## Reading Output Files

### Using Arrow Rust

```rust
use arrow::ipc::reader::StreamReader;
use std::fs::File;

let file = File::open("output_dir/otlp/traces/otlp_traces_20241123_120000_0001.arrow")?;
let reader = StreamReader::try_new(file, None)?;

for batch_result in reader {
    let batch = batch_result?;
    // Process RecordBatch
    println!("Batch schema: {:?}", batch.schema());
    println!("Batch rows: {}", batch.num_rows());
}
```

### Using Python

```python
import pyarrow as pa

# Read Arrow IPC stream
with pa.ipc.open_stream('output_dir/otlp/traces/otlp_traces_20241123_120000_0001.arrow') as reader:
    for batch in reader:
        print(f"Schema: {batch.schema}")
        print(f"Rows: {len(batch)}")
        # Convert to pandas if needed
        df = batch.to_pandas()
```

---

## Common Patterns

### Periodic Export

```rust
use tokio::time::{interval, Duration};

let library = OtlpLibrary::new(Config::default())?;
let mut interval = interval(Duration::from_secs(1));

loop {
    interval.tick().await;
    
    // Collect spans/metrics
    let spans = collect_spans();
    library.export_traces(spans)?;
}
```

### Error Handling

```rust
use otlp_arrow_library::{OtlpLibrary, OtlpError};

match library.export_trace(span) {
    Ok(()) => println!("Exported successfully"),
    Err(OtlpError::Export(e)) => eprintln!("Export failed: {}", e),
    Err(OtlpError::Io(e)) => eprintln!("I/O error: {}", e),
    Err(e) => eprintln!("Other error: {}", e),
}
```

### Graceful Shutdown

```rust
use tokio::signal;

let library = OtlpLibrary::new(Config::default())?;

// Handle shutdown signal
tokio::spawn(async move {
    signal::ctrl_c().await.ok();
    library.shutdown().ok();
});

// Main loop
loop {
    // Process messages
}
```

---

## Troubleshooting

### Output Directory Not Created

**Problem**: Library fails to start with "Failed to create output directory"

**Solution**: Ensure the parent directory exists and has write permissions:
```bash
mkdir -p ./output_dir
chmod 755 ./output_dir
```

### Files Not Being Written

**Problem**: Messages are sent but no files appear

**Solution**: 
1. Check write interval (default 5 seconds) - files are written in batches
2. Call `flush()` to force immediate write
3. Check disk space and permissions

### gRPC Connection Refused

**Problem**: Cannot connect to gRPC service

**Solution**:
1. Verify service is running: `netstat -an | grep 4317`
2. Check firewall settings
3. Verify correct endpoint URL

### High Memory Usage

**Problem**: Library uses too much memory

**Solution**:
1. Reduce write interval to flush more frequently
2. Check buffer size limits
3. Monitor with `RUST_LOG=debug` for buffer statistics

---

## Next Steps

- See [data-model.md](./data-model.md) for entity definitions
- See [contracts/public-api.md](./contracts/public-api.md) for full API documentation
- See [contracts/grpc-api.md](./contracts/grpc-api.md) for gRPC interface details
- See [research.md](./research.md) for implementation details

