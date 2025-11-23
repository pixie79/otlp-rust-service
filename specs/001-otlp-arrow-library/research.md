# Research: OTLP Arrow Flight Library

**Date**: 2024-11-23  
**Feature**: 001-otlp-arrow-library

## Research Questions

### 1. Dual Protocol Support: gRPC with Protobuf and Arrow Flight IPC

**Question**: How to implement support for both gRPC with Protobuf (standard OTLP) and gRPC with Arrow Flight IPC (OTAP) protocols?

**Context**: The specification requires receiving OTLP messages using both gRPC with Protobuf and gRPC with Arrow Flight IPC simultaneously. Clarifications confirmed both protocols should be supported on different ports, with Protobuf on port 4317 and Arrow Flight on configurable port (default 4318).

**Research Findings**:

**Decision**: 
1. Use `opentelemetry-otlp` with gRPC transport for Protobuf protocol (standard OTLP)
2. Use `otel-arrow` Rust crate for Arrow Flight IPC protocol (OTAP) implementation
3. Support both protocols simultaneously on different ports/endpoints
4. Both protocols enabled by default

**Rationale**: 
- `opentelemetry-otlp` crate (0.31) provides built-in gRPC support via `tonic` for standard OTLP Protobuf
- `otel-arrow` Rust crate (https://github.com/open-telemetry/otel-arrow) provides official OpenTelemetry Arrow Flight implementation
- Dual protocol support provides maximum flexibility and backward compatibility
- Standard OTLP port (4317) for Protobuf maintains compatibility with existing OTLP clients
- Separate port for Arrow Flight avoids conflicts and allows independent enable/disable

**Alternatives Considered**:
- Single protocol only: Doesn't meet specification requirements for dual support
- Custom Arrow Flight implementation: Unnecessary, otel-arrow provides official implementation
- Protocol negotiation: More complex, separate ports provide clearer separation

**Implementation Approach**:
1. Use `opentelemetry-otlp` for Protobuf gRPC server on port 4317
2. Integrate `otel-arrow` Rust crate for Arrow Flight IPC server on port 4318 (configurable)
3. Both servers share the same file exporter and batch writer infrastructure
4. Configuration allows independent enable/disable of each protocol
5. Mock service supports both protocols for comprehensive testing

**References**:
- `opentelemetry-otlp` crate documentation
- `otel-arrow` Rust crate: https://github.com/open-telemetry/otel-arrow
- OTLP specification: https://opentelemetry.io/docs/specs/otlp/
- OTAP specification: OpenTelemetry Protocol with Apache Arrow (OTEP)

---

### 2. Arrow IPC Streaming Format for File Output

**Question**: How to write OTLP data to Arrow IPC Streaming format files with batching?

**Context**: Specification requires writing to Arrow IPC Streaming format with configurable batch intervals (default 5 seconds).

**Research Findings**:

**Decision**: Use `arrow::ipc::writer::StreamWriter` with `RecordBatch` batching, following patterns from cap-gl-consumer-rust.

**Rationale**:
- Existing codebase already implements Arrow IPC conversion in `file_exporter.rs`
- `StreamWriter` supports streaming format (multiple RecordBatches per file)
- Batching can be implemented with tokio intervals and in-memory buffers
- File rotation based on size already implemented in existing code

**Alternatives Considered**:
- Arrow IPC File format: Not streaming, requires fixed schema upfront
- Parquet format: More complex, not specified
- Protobuf files: Spec explicitly requires Arrow IPC

**Implementation Approach**:
1. Buffer spans/metrics in memory until batch interval (5s default)
2. Convert buffered data to Arrow `RecordBatch` using existing conversion functions
3. Write RecordBatch to file using `StreamWriter`
4. Rotate files based on size (existing pattern from cap-gl-consumer-rust)
5. Use tokio intervals for batch timing

**References**:
- `cap-gl-consumer-rust/src/otlp/file_exporter.rs:517-648` - Arrow IPC conversion
- Arrow Rust documentation: https://docs.rs/arrow/

---

### 3. Configuration System Design

**Question**: How to implement unified configuration for output directory, write intervals, cleanup, and forwarding?

**Context**: Specification requires configurable output directory, write frequency, cleanup intervals, and optional forwarding settings.

**Research Findings**:

**Decision**: Use `serde` with `serde_yaml` for YAML configuration, environment variables with `OTLP_*` prefix, and programmatic API, following cap-gl-consumer-rust patterns.

**Rationale**:
- Existing codebase uses `serde`/`serde_yaml` pattern (see `config/types.rs`)
- Environment variables provide 12-factor app compatibility
- Programmatic API enables embedded library usage
- YAML provides human-readable configuration for standalone mode

**Alternatives Considered**:
- TOML: Less common, YAML more standard for configuration
- JSON: Less human-readable, no comments
- CLI-only: Too limited for embedded usage

**Implementation Approach**:
1. Define configuration structs with `serde` derives
2. Support loading from YAML file, environment variables, or programmatic
3. Environment variable naming: `OTLP_OUTPUT_DIR`, `OTLP_WRITE_INTERVAL_SECS`, etc.
4. Default values as specified in requirements
5. Validation on load

**References**:
- `cap-gl-consumer-rust/src/config/types.rs` - Configuration structure patterns
- `cap-gl-consumer-rust/src/config/loader.rs` - Configuration loading

---

### 4. Mock Service for End-to-End Testing

**Question**: How to implement a mock OTLP service that supports both gRPC interface and public API method testing?

**Context**: Specification requires mock service for testing both integration paths without external dependencies.

**Research Findings**:

**Decision**: Implement in-memory mock service that:
1. Accepts OTLP messages via gRPC (using `opentelemetry-otlp` test utilities)
2. Accepts OTLP messages via public API methods
3. Validates message format and structure
4. Provides test assertions for received messages

**Rationale**:
- Enables testing without external OTLP collector
- Validates both integration paths (gRPC and API)
- Can be used in integration tests
- Follows testing best practices for library development

**Alternatives Considered**:
- External test OTLP collector: Adds dependency, harder to control
- Mock gRPC server only: Doesn't test public API path
- No mock service: Requires external dependencies for all tests

**Implementation Approach**:
1. Create `MockOtlpService` struct that implements both gRPC server and public API
2. Store received messages in memory for assertions
3. Use `opentelemetry-otlp` test utilities for gRPC mocking
4. Provide test helpers for common assertions
5. Support both metrics and traces

**References**:
- `opentelemetry-otlp` test utilities
- Rust testing best practices

---

### 5. Remote Forwarding Implementation

**Question**: How to implement optional forwarding to remote OTLP endpoints with authentication?

**Context**: Specification requires optional forwarding (disabled by default) with configurable authentication.

**Research Findings**:

**Decision**: Use `opentelemetry-otlp` exporter configured for remote endpoint, with authentication support via standard OTLP headers.

**Rationale**:
- `opentelemetry-otlp` already supports remote export with authentication
- Standard OTLP authentication patterns (headers, tokens)
- Can reuse existing exporter infrastructure
- Forwarding failures should not block local storage

**Alternatives Considered**:
- Custom gRPC client: Unnecessary, opentelemetry-otlp provides this
- HTTP only: Not standard, OTLP uses gRPC
- Synchronous forwarding: Would block message processing

**Implementation Approach**:
1. Configure `opentelemetry-otlp` exporter with remote endpoint URL
2. Support authentication headers (API keys, tokens) via configuration
3. Forward asynchronously to avoid blocking local storage
4. Handle forwarding errors gracefully (log, don't fail)
5. Circuit breaker pattern for repeated failures
6. Support format conversion: Protobuf ↔ Arrow Flight based on configured forwarding protocol
7. Use `otel-arrow` for Arrow Flight forwarding when configured

**Format Conversion**:
- When forwarding Protobuf messages to Arrow Flight endpoint: Convert using `otel-arrow` encoding
- When forwarding Arrow Flight messages to Protobuf endpoint: Convert using `opentelemetry-otlp` protobuf encoding
- Conversion happens transparently during forwarding, preserving all message data
- Conversion errors should be logged and handled gracefully (don't block local storage)

**References**:
- `opentelemetry-otlp` exporter documentation
- `otel-arrow` crate for Arrow Flight encoding/decoding
- OTLP authentication specification

---

### 6. File Cleanup Implementation

**Question**: How to implement configurable file cleanup for traces (600s default) and metrics (1h default)?

**Context**: Specification requires separate cleanup intervals for trace and metric files.

**Research Findings**:

**Decision**: Use tokio background task with interval checking, following pattern from cap-gl-consumer-rust cleanup implementation.

**Rationale**:
- Existing codebase has cleanup pattern in `file_exporter.rs:332-379`
- Tokio intervals provide efficient periodic execution
- Separate intervals for traces/metrics as specified
- File age checking via filesystem metadata

**Alternatives Considered**:
- Synchronous cleanup: Would block main processing
- File-based timestamps: More complex, filesystem metadata sufficient
- Single cleanup interval: Doesn't match spec requirements

**Implementation Approach**:
1. Background tokio task with 60-second check interval
2. Scan output directories for old files
3. Check file modification time against configured age
4. Remove files older than threshold
5. Log cleanup operations

**References**:
- `cap-gl-consumer-rust/src/otlp/file_exporter.rs:332-379` - Cleanup implementation

---

### 7. Python Bindings Implementation

**Question**: How to implement Python bindings for the public API to enable calling from Python projects?

**Context**: Specification requires public API methods to be callable from both Rust projects (native) and Python projects (via Python bindings/FFI).

**Research Findings**:

**Decision**: Use PyO3 crate for Python bindings, providing a Python module that wraps the Rust public API.

**Rationale**: 
- PyO3 is the standard and mature solution for Rust-Python interop
- Provides seamless integration with Python's C API
- Supports async operations, error handling, and memory management
- Well-documented with active community
- Supports building Python wheels for distribution
- Maintains native Rust performance for core operations

**Alternatives Considered**:
- ctypes/CFFI: Lower-level, more manual work, less type safety
- cbindgen + manual Python bindings: More complex, harder to maintain
- Separate Python reimplementation: Duplicates logic, maintenance burden

**Implementation Approach**:
1. Create `src/python/` module with PyO3 bindings
2. Wrap Rust public API types with `#[pyclass]` and `#[pymethods]`
3. Convert Rust types to Python-compatible types (PyO3 handles most conversions)
4. Handle async operations by exposing async methods or using Python's asyncio
5. Build Python package using `maturin` (PyO3's build tool)
6. Distribute as Python wheel (`.whl`) for easy installation
7. Provide Python examples and documentation

**Python API Design**:
- Mirror Rust API structure for consistency
- Use Python naming conventions (snake_case)
- Provide Pythonic error handling (raise exceptions)
- Support both sync and async usage patterns
- Provide type hints for better IDE support

**References**:
- PyO3 documentation: https://pyo3.rs/
- Maturin (build tool): https://github.com/PyO3/maturin
- PyO3 async support: https://pyo3.rs/latest/class/async.html

---

## Summary

All research questions resolved. Implementation will:
1. Use `opentelemetry-otlp` for gRPC receiving (standard OTLP Protobuf on port 4317)
2. Use `otel-arrow` Rust crate for Arrow Flight IPC receiving (OTAP on port 4318, configurable)
3. Support both protocols simultaneously with independent enable/disable configuration
4. Convert received messages to Arrow IPC format for file storage (existing patterns)
5. Use serde-based configuration (YAML, env vars, programmatic)
6. Implement mock service for testing both integration paths and both protocols
7. Use opentelemetry-otlp exporter for remote forwarding (Protobuf) and otel-arrow for Arrow Flight forwarding
8. Support format conversion (Protobuf ↔ Arrow Flight) when forwarding messages in different format than received
9. Use tokio background tasks for file cleanup
10. Use PyO3 for Python bindings to enable Python project integration

All decisions based on existing proven patterns from cap-gl-consumer-rust codebase where applicable. Python bindings follow PyO3 best practices for FFI design. Arrow Flight implementation uses official otel-arrow crate for OTAP protocol compliance. Format conversion enables flexible forwarding regardless of input protocol.

