# Data Model: OTLP Arrow Flight Library

**Date**: 2024-11-23  
**Feature**: 001-otlp-arrow-library

## Entities

### Configuration

Represents all configurable settings for the library.

**Fields**:
- `output_dir: PathBuf` - Output directory for Arrow IPC files (default: `./output_dir`)
- `write_interval_secs: u64` - How frequently to write batches to disk in seconds (default: 5)
- `trace_cleanup_interval_secs: u64` - How frequently to clean old trace files in seconds (default: 600)
- `metric_cleanup_interval_secs: u64` - How frequently to clean old metric files in seconds (default: 3600)
- `protocols: ProtocolConfig` - Protocol configuration (Protobuf and Arrow Flight)
- `forwarding: Option<ForwardingConfig>` - Optional remote forwarding configuration (default: None/disabled)

**Validation Rules**:
- `output_dir` must be a valid path (can be created if doesn't exist)
- `write_interval_secs` must be > 0
- `trace_cleanup_interval_secs` must be > 0
- `metric_cleanup_interval_secs` must be > 0
- All intervals must be reasonable (e.g., < 86400 seconds for cleanup)

**State Transitions**: Configuration is loaded at startup and can be reloaded (requires restart for some settings).

---

### ProtocolConfig

Represents configuration for gRPC protocol support (Protobuf and Arrow Flight).

**Fields**:
- `protobuf_enabled: bool` - Whether gRPC with Protobuf protocol is enabled (default: true)
- `protobuf_port: u16` - Port for gRPC with Protobuf (default: 4317, standard OTLP port)
- `arrow_flight_enabled: bool` - Whether gRPC with Arrow Flight IPC protocol is enabled (default: true)
- `arrow_flight_port: u16` - Port for gRPC with Arrow Flight IPC (default: 4318, configurable)

**Validation Rules**:
- At least one protocol must be enabled
- Ports must be valid (1-65535)
- Ports must be different if both protocols are enabled
- Ports must not conflict with system-reserved ports

**State Transitions**: Protocol configuration can be changed at startup. Enabling/disabling protocols requires service restart.

---

### ForwardingConfig

Represents configuration for optional remote OTLP endpoint forwarding.

**Fields**:
- `enabled: bool` - Whether forwarding is enabled (default: false)
- `endpoint_url: String` - Remote OTLP endpoint URL (required if enabled)
- `protocol: ForwardingProtocol` - Protocol to use for forwarding (Protobuf or Arrow Flight, default: Protobuf)
- `authentication: Option<AuthConfig>` - Authentication configuration (optional)

**Validation Rules**:
- `endpoint_url` must be a valid URL if `enabled` is true
- `endpoint_url` must use `http://` or `https://` scheme
- `protocol` must be either Protobuf or Arrow Flight
- Authentication must be provided if remote endpoint requires it

**State Transitions**: Can be enabled/disabled via configuration, changes require restart.

---

### ForwardingProtocol

Enumeration of supported forwarding protocols.

**Values**:
- `Protobuf` - Standard OTLP gRPC with Protobuf
- `ArrowFlight` - OpenTelemetry Protocol with Apache Arrow (OTAP)

**Usage**: Specifies which protocol to use when forwarding messages to remote endpoints.

---

### AuthConfig

Represents authentication configuration for remote forwarding.

**Fields**:
- `auth_type: String` - Type of authentication (e.g., "api_key", "bearer_token", "basic")
- `credentials: HashMap<String, String>` - Authentication parameters (e.g., token, key, username, password)

**Validation Rules**:
- `auth_type` must be a supported authentication method
- Required credentials must be present based on `auth_type`

**State Transitions**: Loaded with configuration, changes require restart.

---

### OtlpMessage

Represents an OTLP message (trace or metric) received by the library.

**Fields**:
- `message_type: MessageType` - Whether this is a trace or metric
- `data: OtlpData` - The actual telemetry data
- `received_at: SystemTime` - Timestamp when message was received
- `resource: Option<Resource>` - Resource information (optional)

**Validation Rules**:
- `data` must be valid OTLP format
- `received_at` must be current or recent timestamp

**State Transitions**: 
- Created when message is received via gRPC or public API
- Buffered in memory until batch write interval
- Written to file as Arrow IPC format
- Optionally forwarded to remote endpoint

---

### MessageType

Enumeration of OTLP message types.

**Values**:
- `Trace` - OpenTelemetry trace data
- `Metric` - OpenTelemetry metric data

---

### OtlpData

Represents the actual OTLP telemetry data (traces or metrics).

**For Traces**:
- `spans: Vec<SpanData>` - Collection of span data

**For Metrics**:
- `resource_metrics: ResourceMetrics` - Resource-scoped metrics data

**Validation Rules**:
- Must match OTLP protocol specification
- Spans must have valid trace/span IDs
- Metrics must have valid metric names and values

---

### BatchBuffer

Represents in-memory buffer for batching messages before writing to disk.

**Fields**:
- `traces: Vec<SpanData>` - Buffered trace spans
- `metrics: Vec<ResourceMetrics>` - Buffered metrics
- `last_write: SystemTime` - Timestamp of last batch write
- `write_interval: Duration` - Configured write interval

**Validation Rules**:
- Buffer size should be bounded to prevent memory exhaustion
- Write interval must match configuration

**State Transitions**:
- Messages added to buffer when received
- Buffer flushed to disk when interval elapsed or buffer full
- Buffer cleared after successful write

---

### OutputFile

Represents a file written to disk containing Arrow IPC Streaming format data.

**Fields**:
- `file_path: PathBuf` - Full path to the file
- `file_type: FileType` - Whether this is a trace or metric file
- `created_at: SystemTime` - File creation timestamp
- `size_bytes: u64` - Current file size in bytes
- `record_batch_count: u64` - Number of RecordBatches written

**Validation Rules**:
- File must be valid Arrow IPC Streaming format
- File path must be in correct subdirectory (`{OUTPUT_DIR}/otlp/traces` or `{OUTPUT_DIR}/otlp/metrics`)
- File must be readable by standard Arrow libraries

**State Transitions**:
- Created when first batch is written
- Appended to when additional batches are written
- Rotated when file size exceeds maximum
- Deleted when age exceeds cleanup interval

---

### FileType

Enumeration of output file types.

**Values**:
- `Trace` - File contains trace data
- `Metric` - File contains metric data

---

### MockServiceState

Represents the internal state of the mock OTLP service for testing.

**Fields**:
- `received_traces: Vec<SpanData>` - Traces received via mock service
- `received_metrics: Vec<ResourceMetrics>` - Metrics received via mock service
- `grpc_calls: u64` - Count of gRPC calls received
- `api_calls: u64` - Count of public API calls received

**Validation Rules**:
- State should be resettable for test isolation
- All received messages should be accessible for assertions

**State Transitions**:
- Updated when messages received via gRPC or API
- Reset between tests
- Queried for test assertions

---

## Relationships

- **Configuration** → **ForwardingConfig** (optional, one-to-one)
- **ForwardingConfig** → **AuthConfig** (optional, one-to-one)
- **ForwardingConfig** → **ForwardingProtocol** (required, one-to-one)
- **OtlpMessage** → **OtlpData** (required, one-to-one)
- **BatchBuffer** → **OtlpMessage** (one-to-many, contains buffered messages)
- **OutputFile** → **BatchBuffer** (one-to-many, contains written batches)
- **MockServiceState** → **OtlpMessage** (one-to-many, tracks received messages)
- **Forwarding** → **FormatConverter** (conditional, used when input format differs from forwarding format)

---

### FormatConverter

Represents the format conversion component that converts OTLP messages between Protobuf and Arrow Flight formats.

**Purpose**: Enable flexible forwarding by converting messages to the configured forwarding format when input format differs.

**Conversion Directions**:
- Protobuf → Arrow Flight: Uses `otel-arrow` crate for encoding
- Arrow Flight → Protobuf: Uses `opentelemetry-otlp` protobuf encoding

**Validation Rules**:
- Conversion must preserve all message data, attributes, and metadata
- Conversion errors must be logged and handled gracefully
- Conversion failures should not block local storage operations

**State Transitions**: Conversion is performed on-demand during forwarding operations when format mismatch is detected.

---

## Data Flow

1. **Ingestion**: OTLP messages received via gRPC or public API → `OtlpMessage`
2. **Buffering**: `OtlpMessage` → `BatchBuffer` (traces or metrics)
3. **Writing**: `BatchBuffer` → `OutputFile` (Arrow IPC format) at configured interval
4. **Forwarding** (optional): `OtlpMessage` → Format conversion (if needed) → Remote endpoint via `ForwardingConfig`
5. **Cleanup**: `OutputFile` deleted when age exceeds cleanup interval

**Format Conversion Flow** (during forwarding):
- If input format (Protobuf/Arrow Flight) differs from configured forwarding format → Convert using `FormatConverter`
- Conversion preserves all message data, attributes, and metadata
- Conversion errors are logged and handled gracefully (don't block local storage)

---

## Arrow IPC Schema

### Trace RecordBatch Schema

```rust
Schema {
    fields: [
        Field::new("trace_id", DataType::Binary, false),
        Field::new("span_id", DataType::Binary, false),
        Field::new("parent_span_id", DataType::Binary, true),
        Field::new("name", DataType::Utf8, false),
        Field::new("kind", DataType::Int32, false),
        Field::new("start_time_unix_nano", DataType::UInt64, false),
        Field::new("end_time_unix_nano", DataType::UInt64, false),
        Field::new("status_code", DataType::Int32, false),
        Field::new("status_message", DataType::Utf8, true),
        Field::new("attributes", DataType::Utf8, true), // JSON-encoded
    ]
}
```

### Metric RecordBatch Schema

```rust
Schema {
    fields: [
        Field::new("metric_name", DataType::Utf8, false),
        Field::new("value", DataType::Float64, false),
        Field::new("timestamp_unix_nano", DataType::UInt64, false),
        Field::new("metric_type", DataType::Utf8, false),
        Field::new("attributes", DataType::Utf8, true), // JSON-encoded
    ]
}
```

**Note**: These schemas are based on the conversion patterns from `cap-gl-consumer-rust/src/otlp/file_exporter.rs`. The actual schema may be refined during implementation to better match OTLP data structures.

