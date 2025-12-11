# Architecture Documentation

**Project**: OTLP Rust Service  
**Version**: 0.5.0  
**Last Updated**: 2025-01-27

## Table of Contents

1. [System Overview](#system-overview)
2. [High-Level Architecture](#high-level-architecture)
3. [Data Flow](#data-flow)
4. [Component Interactions](#component-interactions)
5. [Key Design Decisions](#key-design-decisions)
6. [Technology Stack](#technology-stack)
7. [Deployment Architecture](#deployment-architecture)
8. [Extension Points](#extension-points)

---

## System Overview

The OTLP Rust Service is a cross-platform library and standalone service for receiving OpenTelemetry Protocol (OTLP) messages via gRPC and writing them to local files in Arrow IPC Streaming format. The system supports both embedded library usage and standalone service mode, with optional remote forwarding capabilities.

### Core Capabilities

- **Dual Protocol Support**: Simultaneous support for gRPC Protobuf (standard OTLP) and gRPC Arrow Flight (OTAP) on different ports
- **Arrow IPC Storage**: Efficient storage of telemetry data in Arrow IPC Streaming format with automatic file rotation
- **Batch Writing**: Configurable write intervals for efficient disk I/O
- **File Cleanup**: Automatic cleanup of old trace and metric files based on configurable retention intervals
- **Public API**: Embedded library mode with programmatic API for Rust and Python
- **Optional Forwarding**: Forward messages to remote OTLP endpoints with automatic format conversion
- **Circuit Breaker**: Automatic failure handling with circuit breaker pattern for forwarding
- **OpenTelemetry SDK Integration**: Built-in `PushMetricExporter` and `SpanExporter` implementations

---

## High-Level Architecture

```
┌─────────────────────────────────────────────────────────────────┐
│                      OTLP Rust Service                           │
├─────────────────────────────────────────────────────────────────┤
│                                                                  │
│  ┌──────────────┐         ┌──────────────┐                      │
│  │   gRPC       │         │   gRPC      │                      │
│  │  Protobuf    │         │ Arrow Flight│                      │
│  │  Server      │         │   Server     │                      │
│  │  (Port 4317) │         │  (Port 4318)│                      │
│  └──────┬───────┘         └──────┬───────┘                      │
│         │                        │                              │
│         └────────────┬───────────┘                              │
│                      │                                          │
│         ┌────────────▼──────────┐                               │
│         │   OtlpFileExporter    │                               │
│         │  (Format Conversion)   │                               │
│         └────────────┬──────────┘                               │
│                      │                                          │
│         ┌────────────▼──────────┐                               │
│         │    BatchBuffer        │                               │
│         │  (In-Memory Buffer)   │                               │
│         └────────────┬──────────┘                               │
│                      │                                          │
│         ┌────────────▼──────────┐                               │
│         │   Arrow IPC Writer    │                               │
│         │  (File Storage)       │                               │
│         └──────────────────────┘                               │
│                                                                  │
│  ┌──────────────────────────────────────────────┐              │
│  │         Optional Remote Forwarding            │              │
│  │  ┌──────────────────────────────────────┐   │              │
│  │  │   OtlpForwarder                       │   │              │
│  │  │  ┌────────────────────────────────┐   │   │              │
│  │  │  │  Circuit Breaker               │   │   │              │
│  │  │  │  (Failure Handling)            │   │   │              │
│  │  │  └────────────────────────────────┘   │   │              │
│  │  │  ┌────────────────────────────────┐   │   │              │
│  │  │  │  FormatConverter              │   │   │              │
│  │  │  │  (Protobuf ↔ Arrow Flight)    │   │   │              │
│  │  │  └────────────────────────────────┘   │   │              │
│  │  └──────────────────────────────────────┘   │              │
│  └──────────────────────────────────────────────┘              │
│                                                                  │
│  ┌──────────────────────────────────────────────┐              │
│  │         Public API (Embedded Mode)          │              │
│  │  ┌──────────────────────────────────────┐   │              │
│  │  │   OtlpLibrary                        │   │              │
│  │  │  (Rust & Python Bindings)            │   │              │
│  │  └──────────────────────────────────────┘   │              │
│  └──────────────────────────────────────────────┘              │
│                                                                  │
└─────────────────────────────────────────────────────────────────┘
```

### Architecture Layers

1. **Ingestion Layer**: gRPC servers (Protobuf and Arrow Flight) receive OTLP messages
2. **Processing Layer**: Format conversion, buffering, and batching
3. **Storage Layer**: Arrow IPC file writing with rotation and cleanup
4. **Forwarding Layer** (Optional): Remote forwarding with circuit breaker protection
5. **API Layer**: Public API for embedded usage (Rust and Python)

---

## Data Flow

### Trace Data Flow

```
Client SDK/Application
    │
    ├─→ gRPC Protobuf (Port 4317)
    │       │
    │       └─→ TraceServiceImpl
    │               │
    │               └─→ OtlpFileExporter.export_trace()
    │                       │
    │                       └─→ BatchBuffer.add_trace()
    │                               │
    │                               └─→ [Buffered in Memory]
    │                                       │
    │                                       └─→ [Periodic Write]
    │                                               │
    │                                               └─→ Arrow IPC Writer
    │                                                       │
    │                                                       └─→ File: traces/*.arrow
    │
    ├─→ gRPC Arrow Flight (Port 4318)
    │       │
    │       └─→ OtlpArrowFlightServer
    │               │
    │               └─→ OtlpFileExporter.export_trace()
    │                       │
    │                       └─→ [Same as Protobuf path]
    │
    └─→ Public API (Rust/Python)
            │
            └─→ OtlpLibrary.export_trace()
                    │
                    └─→ [Same as gRPC path]
```

### Metrics Data Flow

```
Client SDK/Application
    │
    ├─→ gRPC Protobuf (Port 4317)
    │       │
    │       └─→ MetricsServiceImpl
    │               │
    │               └─→ OtlpFileExporter.export_metrics()
    │                       │
    │                       ├─→ [Convert Protobuf → Arrow RecordBatch]
    │                       │
    │                       └─→ BatchBuffer.add_metrics_protobuf()
    │                               │
    │                               └─→ [Buffered in Memory]
    │                                       │
    │                                       └─→ [Periodic Write]
    │                                               │
    │                                               └─→ Arrow IPC Writer
    │                                                       │
    │                                                       └─→ File: metrics/*.arrow
    │
    ├─→ gRPC Arrow Flight (Port 4318)
    │       │
    │       └─→ OtlpArrowFlightServer
    │               │
    │               └─→ OtlpFileExporter.export_metrics()
    │                       │
    │                       └─→ [Same as Protobuf path]
    │
    └─→ Public API (Rust/Python)
            │
            └─→ OtlpLibrary.export_metrics()
                    │
                    └─→ [Same as gRPC path]
```

### Forwarding Data Flow (Optional)

```
BatchBuffer (on write)
    │
    └─→ OtlpForwarder.forward_traces() / forward_metrics()
            │
            ├─→ Circuit Breaker Check
            │       │
            │       ├─→ [Open] → Reject immediately
            │       ├─→ [HalfOpen] → Allow one test request
            │       └─→ [Closed] → Proceed normally
            │
            └─→ FormatConverter
                    │
                    ├─→ [If target is Protobuf] → Convert Arrow → Protobuf
                    └─→ [If target is Arrow Flight] → Keep Arrow format
                            │
                            └─→ HTTP/gRPC Client
                                    │
                                    └─→ Remote OTLP Endpoint
```

---

## Component Interactions

### Core Components

#### 1. OtlpGrpcServer (`src/otlp/server.rs`)
- **Purpose**: gRPC server for Protobuf OTLP messages
- **Responsibilities**:
  - Receives `ExportTraceServiceRequest` and `ExportMetricsServiceRequest`
  - Delegates to `OtlpFileExporter` for processing
  - Handles gRPC request/response lifecycle
- **Dependencies**: `OtlpFileExporter`, `tonic` gRPC framework

#### 2. OtlpArrowFlightServer (`src/otlp/server_arrow.rs`)
- **Purpose**: gRPC server for Arrow Flight IPC (OTAP) messages
- **Responsibilities**:
  - Receives Arrow Flight `FlightData` streams
  - Extracts Arrow RecordBatches from Flight messages
  - Delegates to `OtlpFileExporter` for processing
- **Dependencies**: `OtlpFileExporter`, `arrow-flight` crate

#### 3. OtlpFileExporter (`src/otlp/exporter.rs`)
- **Purpose**: Central exporter for all OTLP data
- **Responsibilities**:
  - Format conversion (Protobuf ↔ Arrow)
  - Buffering via `BatchBuffer`
  - Coordinating writes and cleanup
  - Optional forwarding via `OtlpForwarder`
- **Dependencies**: `BatchBuffer`, `OtlpForwarder` (optional), format converters

#### 4. BatchBuffer (`src/otlp/batch_writer.rs`)
- **Purpose**: In-memory buffer for batching OTLP messages
- **Responsibilities**:
  - Thread-safe buffering of traces and metrics
  - Capacity limit enforcement
  - Providing batches for periodic writes
- **Concurrency**: Uses `Arc<Mutex<Vec<T>>>` for thread-safe access
- **Dependencies**: None (core data structure)

#### 5. OtlpForwarder (`src/otlp/forwarder.rs`)
- **Purpose**: Remote forwarding with failure handling
- **Responsibilities**:
  - Format conversion for target protocol
  - Circuit breaker pattern for failure handling
  - HTTP/gRPC client management
  - Authentication handling
- **Dependencies**: `FormatConverter`, `reqwest` HTTP client, circuit breaker logic

#### 6. OtlpLibrary (`src/api/public.rs`)
- **Purpose**: Public API for embedded usage
- **Responsibilities**:
  - High-level API for Rust and Python applications
  - Background task management (writes, cleanup)
  - Configuration management
  - Dashboard server coordination
- **Dependencies**: `OtlpFileExporter`, `BatchBuffer`, `DashboardServer`

### Component Communication Patterns

1. **Request-Response**: gRPC servers receive requests and delegate to exporters
2. **Async Buffering**: Messages are buffered asynchronously, written periodically
3. **Event-Driven**: Background tasks trigger writes and cleanup based on intervals
4. **Circuit Breaker**: Forwarding uses circuit breaker pattern to handle failures gracefully

---

## Key Design Decisions

### 1. Dual Protocol Support

**Decision**: Support both gRPC Protobuf and Arrow Flight simultaneously on different ports.

**Rationale**:
- Protobuf is the standard OTLP format, ensuring compatibility
- Arrow Flight provides better performance for high-throughput scenarios
- Different ports allow clients to choose the appropriate protocol

**Implementation**: Separate server implementations (`OtlpGrpcServer` and `OtlpArrowFlightServer`) that share the same `OtlpFileExporter`.

### 2. Batch Buffering

**Decision**: Buffer messages in memory and write in batches at configurable intervals.

**Rationale**:
- Reduces disk I/O overhead
- Improves throughput for high-volume scenarios
- Configurable intervals allow tuning for different use cases

**Implementation**: `BatchBuffer` uses `Arc<Mutex<Vec<T>>>` for thread-safe buffering with capacity limits.

### 3. Arrow IPC Storage Format

**Decision**: Store telemetry data in Arrow IPC Streaming format.

**Rationale**:
- Efficient columnar format for analytics
- Cross-language compatibility
- Efficient compression and querying capabilities

**Implementation**: All data is converted to Arrow RecordBatches and written as Arrow IPC Streaming files.

### 4. Circuit Breaker for Forwarding

**Decision**: Implement circuit breaker pattern for remote forwarding failures.

**Rationale**:
- Prevents cascading failures when remote endpoints are down
- Reduces unnecessary network traffic during outages
- Automatic recovery when remote endpoint recovers

**Implementation**: Three-state circuit breaker (Closed, Open, HalfOpen) with configurable thresholds and timeouts.

### 5. Format Conversion

**Decision**: Automatic format conversion between Protobuf and Arrow Flight.

**Rationale**:
- Allows clients to use either protocol regardless of storage format
- Enables forwarding to endpoints with different protocol requirements
- Maintains data fidelity during conversion

**Implementation**: `FormatConverter` handles bidirectional conversion between Protobuf and Arrow formats.

### 6. Thread-Safe Concurrency Model

**Decision**: Use `Arc<Mutex<T>>` for shared state and async/await for concurrency.

**Rationale**:
- Rust's ownership system prevents data races
- Async/await provides efficient I/O concurrency
- Mutex ensures thread-safe access to shared buffers

**Implementation**: All shared state (buffers, circuit breaker state) uses `Arc<Mutex<T>>` or `Arc<RwLock<T>>` where appropriate.

### 7. Embedded vs Standalone Modes

**Decision**: Support both embedded library usage and standalone service mode.

**Rationale**:
- Embedded mode allows integration into existing applications
- Standalone mode provides a ready-to-use service
- Same core components support both modes

**Implementation**: `OtlpLibrary` provides public API for embedded usage, while `main.rs` provides standalone service.

---

## Technology Stack

### Core Dependencies

- **Rust**: Edition 2024, latest stable version
- **Tokio**: Async runtime (`tokio` 1.35+)
- **OpenTelemetry**: OTLP protocol support (`opentelemetry` 0.31, `opentelemetry-sdk` 0.31)
- **Arrow**: Columnar data format (`arrow` 57, `arrow-flight` 57)
- **gRPC**: `tonic` 0.14 (Protobuf), `arrow-flight` (Arrow Flight)
- **HTTP Client**: `reqwest` 0.11 (for forwarding)
- **Serialization**: `serde` 1.0, `prost` 0.14 (Protobuf)

### Python Integration

- **PyO3**: Python bindings (`pyo3` 0.20)
- **Maturin**: Python package building

### Testing & Development

- **Testing**: `tokio-test` 0.4 (async testing utilities)
- **Benchmarking**: `criterion` 0.5 (performance benchmarks)
- **Mocking**: `wiremock` 0.6 (HTTP mocking for tests)

### Security

- **secrecy**: Secure string types for credential storage
- **url**: Comprehensive URL parsing and validation

---

## Deployment Architecture

### Standalone Service Mode

```
┌─────────────────────────────────────┐
│   otlp-arrow-service (Binary)       │
│                                     │
│  ┌───────────────────────────────┐  │
│  │  Configuration Loader         │  │
│  │  (YAML, Environment, API)    │  │
│  └──────────────┬────────────────┘  │
│                 │                     │
│  ┌──────────────▼────────────────┐  │
│  │  OtlpLibrary                  │  │
│  │  ┌──────────────────────────┐ │  │
│  │  │  gRPC Servers           │ │  │
│  │  │  (Protobuf + Arrow)      │ │  │
│  │  └──────────────────────────┘ │  │
│  │  ┌──────────────────────────┐ │  │
│  │  │  Background Tasks        │ │  │
│  │  │  (Writes, Cleanup)      │ │  │
│  │  └──────────────────────────┘ │  │
│  └────────────────────────────────┘  │
│                 │                     │
│  ┌──────────────▼────────────────┐  │
│  │  File System                  │  │
│  │  ./output_dir/otlp/          │  │
│  │    ├── traces/*.arrow        │  │
│  │    └── metrics/*.arrow       │  │
│  └────────────────────────────────┘  │
└─────────────────────────────────────┘
```

### Embedded Library Mode

```
┌─────────────────────────────────────┐
│   Application (Rust/Python)         │
│                                     │
│  ┌───────────────────────────────┐ │
│  │  OtlpLibrary (Public API)     │ │
│  │  - export_trace()              │ │
│  │  - export_metrics()            │ │
│  │  - flush()                     │ │
│  └──────────────┬──────────────────┘ │
│                 │                     │
│  ┌──────────────▼────────────────┐  │
│  │  Same Core Components         │  │
│  │  (OtlpFileExporter, etc.)     │  │
│  └────────────────────────────────┘  │
└─────────────────────────────────────┘
```

### Runtime Characteristics

- **Concurrency**: Async/await with Tokio runtime
- **Memory**: Bounded buffers with configurable capacity limits
- **Disk I/O**: Batched writes at configurable intervals
- **Network**: Async HTTP/gRPC clients for forwarding
- **Error Handling**: Circuit breaker pattern for forwarding failures

---

## Extension Points

### 1. Custom Exporters

**Location**: `src/otlp/exporter.rs`

The `OtlpFileExporter` can be extended to support additional export formats or destinations. The exporter interface is designed to be composable.

### 2. Format Converters

**Location**: `src/otlp/converter.rs`

New format conversions can be added by extending the `FormatConverter` trait or adding new conversion methods.

### 3. Authentication Methods

**Location**: `src/config/types.rs` (`AuthConfig`)

New authentication methods can be added by extending the `AuthConfig` enum and updating the forwarding logic.

### 4. Storage Backends

**Location**: `src/otlp/exporter.rs` (`OtlpFileExporter`)

The storage layer can be extended to support additional backends (e.g., object storage, databases) by implementing a storage trait.

### 5. Circuit Breaker Strategies

**Location**: `src/otlp/forwarder.rs` (`CircuitBreaker`)

Different circuit breaker strategies (e.g., sliding window, adaptive thresholds) can be implemented by extending the `CircuitBreaker` struct.

### 6. Dashboard Extensions

**Location**: `dashboard/` and `src/dashboard/`

The dashboard can be extended with new visualizations, filters, or data sources.

---

## Related Documentation

- [README.md](../README.md): User-facing documentation and quick start guide
- [docs/metrics-flow-diagram.md](metrics-flow-diagram.md): Detailed metrics import/export flow
- [specs/](../specs/): Feature specifications and implementation plans

---

## References

- [OpenTelemetry Protocol Specification](https://github.com/open-telemetry/opentelemetry-specification)
- [Apache Arrow Documentation](https://arrow.apache.org/docs/)
- [Circuit Breaker Pattern](https://martinfowler.com/bliki/CircuitBreaker.html)
- [Tokio Async Runtime](https://tokio.rs/)
