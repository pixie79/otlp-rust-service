# gRPC API Contract: OTLP Arrow Flight Library

**Date**: 2024-11-23  
**Feature**: 001-otlp-arrow-library

## Overview

The library implements the OpenTelemetry Protocol (OTLP) gRPC services for receiving traces and metrics. The library supports two protocols simultaneously:
1. **gRPC with Protobuf** (standard OTLP) - Port 4317 (default, configurable)
2. **gRPC with Arrow Flight IPC** (OTAP) - Port 4318 (default, configurable)

Both protocols can be enabled/disabled independently via configuration. Both protocols share the same file exporter and batch writer infrastructure.

## Protocol 1: gRPC with Protobuf (Standard OTLP)

### Service Definition

The library implements the standard OTLP gRPC service as defined in the OpenTelemetry Protocol specification using Protobuf.

**Port**: 4317 (default, configurable)  
**Service**: `opentelemetry.proto.collector.trace.v1.TraceService`  
**Service**: `opentelemetry.proto.collector.metrics.v1.MetricsService`

**Implementation**: Uses `opentelemetry-otlp` crate with `tonic` gRPC framework.

---

### Trace Service

#### `Export(ExportTraceServiceRequest) -> ExportTraceServiceResponse`

Receives trace data via gRPC.

**Request** (`ExportTraceServiceRequest`):
```protobuf
message ExportTraceServiceRequest {
  repeated opentelemetry.proto.trace.v1.ResourceSpans resource_spans = 1;
}
```

**Response** (`ExportTraceServiceResponse`):
```protobuf
message ExportTraceServiceResponse {
  opentelemetry.proto.collector.trace.v1.ExportTracePartialSuccess partial_success = 1;
}
```

**Behavior**:
1. Receives trace data in OTLP protobuf format
2. Converts to internal `SpanData` format
3. Buffers for batch writing
4. Returns success response

**Errors**:
- `INVALID_ARGUMENT` - Malformed request
- `INTERNAL` - Processing error
- `UNAVAILABLE` - Service temporarily unavailable

**Example gRPC Call**:
```rust
use opentelemetry_proto::tonic::collector::trace::v1::trace_service_client::TraceServiceClient;
use opentelemetry_proto::tonic::collector::trace::v1::ExportTraceServiceRequest;

let mut client = TraceServiceClient::connect("http://[::1]:4317").await?;
let request = ExportTraceServiceRequest { /* ... */ };
let response = client.export(request).await?;
```

---

### Metrics Service (Protobuf)

#### `Export(ExportMetricsServiceRequest) -> ExportMetricsServiceResponse`

Receives metrics data via gRPC.

**Request** (`ExportMetricsServiceRequest`):
```protobuf
message ExportMetricsServiceRequest {
  repeated opentelemetry.proto.metrics.v1.ResourceMetrics resource_metrics = 1;
}
```

**Response** (`ExportMetricsServiceResponse`):
```protobuf
message ExportMetricsServiceResponse {
  opentelemetry.proto.collector.metrics.v1.ExportMetricsPartialSuccess partial_success = 1;
}
```

**Behavior**:
1. Receives metrics data in OTLP protobuf format
2. Converts to internal `ResourceMetrics` format
3. Buffers for batch writing
4. Returns success response

**Errors**:
- `INVALID_ARGUMENT` - Malformed request
- `INTERNAL` - Processing error
- `UNAVAILABLE` - Service temporarily unavailable

**Example gRPC Call**:
```rust
use opentelemetry_proto::tonic::collector::metrics::v1::metrics_service_client::MetricsServiceClient;
use opentelemetry_proto::tonic::collector::metrics::v1::ExportMetricsServiceRequest;

let mut client = MetricsServiceClient::connect("http://[::1]:4317").await?;
let request = ExportMetricsServiceRequest { /* ... */ };
let response = client.export(request).await?;
```

---

## Endpoint Configuration (Protobuf)

### Default Endpoints

- **Traces**: `0.0.0.0:4317` (OTLP gRPC standard port)
- **Metrics**: `0.0.0.0:4317` (same port, different service)

### Custom Endpoints

Endpoints can be configured via:
- Configuration file (YAML)
- Environment variables (`OTLP_GRPC_PORT`)
- Programmatic API

---

## Authentication

The gRPC service supports standard gRPC authentication mechanisms:

### No Authentication (Default)

Service accepts connections without authentication.

### API Key Authentication

Authentication via gRPC metadata headers:
- Header: `x-api-key: <api-key>`
- Validated against configured API keys

### Bearer Token Authentication

Authentication via gRPC metadata headers:
- Header: `authorization: Bearer <token>`
- Token validated against configured token store

### mTLS (Mutual TLS)

Authentication via TLS client certificates:
- Server validates client certificate
- Certificate must be in trusted CA chain

**Configuration**:
```yaml
grpc:
  tls:
    enabled: true
    cert_file: /path/to/cert.pem
    key_file: /path/to/key.pem
    ca_file: /path/to/ca.pem
```

---

## Protocol 2: gRPC with Arrow Flight IPC (OTAP)

### Service Definition

The library implements the OpenTelemetry Protocol with Apache Arrow (OTAP) using Arrow Flight IPC over gRPC.

**Port**: 4318 (default, configurable)  
**Implementation**: Uses `otel-arrow` Rust crate for Arrow Flight IPC protocol implementation.

### Arrow Flight Service

The Arrow Flight service follows the OTAP specification, which uses Apache Arrow's Flight protocol for efficient columnar data transfer.

**Service**: Arrow Flight gRPC service (as defined by otel-arrow crate)

**Key Features**:
- Columnar data representation for efficient compression
- Dictionary encoding for repeated attribute values
- Resource and Scope separation for reduced redundancy
- Streaming support for high-throughput scenarios

**Behavior**:
1. Receives OTLP data encoded in Arrow Flight IPC format
2. Converts Arrow Flight messages to internal format
3. Buffers for batch writing (same as Protobuf protocol)
4. Returns success response

**Errors**:
- Arrow Flight protocol errors (handled by otel-arrow crate)
- Internal processing errors
- Service unavailability

**Example Arrow Flight Client Call**:
```rust
// Arrow Flight client usage (via otel-arrow crate)
// Implementation details depend on otel-arrow API
```

**References**:
- OTAP specification: OpenTelemetry Protocol with Apache Arrow (OTEP)
- otel-arrow Rust crate: https://github.com/open-telemetry/otel-arrow
- Arrow Flight specification: https://arrow.apache.org/docs/format/Flight.html

---

## Protocol Compliance

The implementation follows:
- **OTLP Specification**: https://opentelemetry.io/docs/specs/otlp/
- **gRPC Transport**: https://opentelemetry.io/docs/specs/otlp/#otlpgrpc
- **Protobuf Schema**: https://github.com/open-telemetry/opentelemetry-proto
- **OTAP Specification**: OpenTelemetry Protocol with Apache Arrow (OTEP)
- **Arrow Flight**: https://arrow.apache.org/docs/format/Flight.html

---

## Error Handling

### gRPC Status Codes

- `OK` - Request processed successfully
- `INVALID_ARGUMENT` - Malformed request data
- `FAILED_PRECONDITION` - Service not ready (e.g., output directory unavailable)
- `RESOURCE_EXHAUSTED` - Rate limit or buffer full
- `INTERNAL` - Internal processing error
- `UNAVAILABLE` - Service temporarily unavailable

### Error Details

Errors include detailed messages in gRPC status:
```rust
Status::internal("Failed to write batch: disk full")
    .with_details("output_dir", "/path/to/output")
```

---

## Performance Characteristics

### Throughput

- Target: 1000+ messages per second
- Measured at gRPC service entry point

### Latency

- p95 latency: < 100ms from request receipt to batch write initiation
- Does not include disk I/O completion time

### Concurrency

- Service handles concurrent requests
- Internal buffering and batching prevent blocking

---

## Mock Service gRPC Interface

The mock service implements both gRPC protocols (Protobuf and Arrow Flight) for comprehensive testing:

### `MockTraceService`

Implements `TraceService` for testing:
- Accepts same `ExportTraceServiceRequest`
- Returns same `ExportTraceServiceResponse`
- Tracks received requests for assertions

### `MockMetricsService`

Implements `MetricsService` for testing:
- Accepts same `ExportMetricsServiceRequest`
- Returns same `ExportMetricsServiceResponse`
- Tracks received requests for assertions

### Usage in Tests

```rust
use otlp_arrow_library::mock::MockOtlpService;

let mock_service = MockOtlpService::new();
let (protobuf_addr, arrow_flight_addr) = mock_service.start().await?;

// Test Protobuf gRPC client
let mut protobuf_client = TraceServiceClient::connect(protobuf_addr).await?;
// ... make Protobuf requests ...

// Test Arrow Flight client (via otel-arrow)
// ... make Arrow Flight requests ...

// Assert received messages (from both protocols)
mock_service.assert_traces_received(10).await?;
```

---

## Versioning

The gRPC API follows OTLP protocol versioning:
- Protocol version: OTLP v1.0+
- Backward compatible with OTLP v1.0 clients
- Future protocol versions will be supported via version negotiation

