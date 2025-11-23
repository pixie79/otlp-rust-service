# Implementation Plan: OTLP Arrow Flight Library

**Branch**: `001-otlp-arrow-library` | **Date**: 2024-11-23 | **Spec**: [spec.md](./spec.md)
**Input**: Feature specification from `/specs/001-otlp-arrow-library/spec.md`

**Note**: This template is filled in by the `/speckit.plan` command. See `.specify/templates/commands/plan.md` for the execution workflow.

## Summary

Build a cross-platform Rust library that receives OTLP (OpenTelemetry Protocol) messages via both gRPC with Protobuf (standard OTLP) and gRPC with Arrow Flight IPC (OTAP), writes them to local files in Arrow IPC Streaming format, and optionally forwards them to remote endpoints with automatic format conversion. The library must support both standalone service mode and embedded library usage with public API methods callable from both Rust and Python projects. Based on existing OTLP implementation patterns from cap-gl-consumer-rust codebase, with Arrow Flight support via otel-arrow Rust crate and format conversion for flexible forwarding.

## Technical Context

**Language/Version**: Rust stable channel (latest stable, minimum 1.75+)  
**Primary Dependencies**: 
- `opentelemetry` (0.31) - OTLP protocol support
- `opentelemetry-sdk` (0.31) - SDK implementation
- `opentelemetry-otlp` (0.31) - OTLP exporter with gRPC Protobuf support
- `opentelemetry-proto` (0.31) - OTLP protobuf definitions
- `otel-arrow` (latest) - Arrow Flight IPC protocol implementation (OTAP) and format conversion
- `arrow` (57) - Arrow IPC format support
- `arrow-array` (57) - Arrow array types
- `tokio` (1.35+) - Async runtime
- `tonic` (0.14) - gRPC framework (must match opentelemetry-proto version)
- `serde` (1.0) - Serialization
- `serde_yaml` (0.9) - YAML configuration
- `anyhow` (1.0) - Error handling
- `tracing` (0.1) - Structured logging
- `pyo3` (0.20+) - Python bindings (FFI for Python support)

**Storage**: Local filesystem (Arrow IPC Streaming format files)  
**Testing**: `cargo test` with unit, integration, and contract tests. Mock service for end-to-end testing supporting both gRPC protocols. Python tests using pytest for Python bindings.  
**Target Platform**: Cross-platform (Windows, Linux, macOS)  
**Project Type**: Rust library (can be used standalone or embedded) with Python bindings  
**Performance Goals**: 
- Receive and store 1000+ OTLP messages per second
- p95 latency < 100ms from message receipt to batch write initiation
- Support both protocols simultaneously without performance degradation
- Format conversion during forwarding should not significantly impact throughput

**Constraints**: 
- Minimum 85% code coverage per file (constitution requirement)
- TDD mandatory for all new features
- All code must pass `cargo clippy` with no warnings
- Cross-platform compatibility required
- Both protocols must be supported simultaneously
- Format conversion must preserve data integrity and ordering

**Scale/Scope**: 
- Library must handle high-throughput OTLP ingestion (1000+ msg/sec)
- Support both Protobuf and Arrow Flight protocols concurrently
- Configurable output directory, write intervals, and cleanup schedules
- Optional remote forwarding with configurable protocol per endpoint and automatic format conversion

## Constitution Check

*GATE: Must pass before Phase 0 research. Re-check after Phase 1 design.*

Verify compliance with OTLP Rust Service Constitution principles:

- **Code Quality (I)**: ✅ Design follows Rust best practices with clear module separation, SOLID principles, and comprehensive documentation. Complexity managed through modular architecture (config, otlp, api, mock modules). Format conversion logic isolated in dedicated conversion module. All public APIs documented with examples.

- **Testing Standards (II)**: ✅ Testing strategy defined with TDD mandatory approach. Test types planned: unit tests (fast, isolated), integration tests (gRPC endpoints, file I/O, format conversion), contract tests (OTLP protocol compliance), and performance tests (latency, throughput, conversion overhead). Mock service enables end-to-end testing of both protocols and format conversion scenarios. Minimum 85% code coverage per file enforced.

- **User Experience Consistency (III)**: ✅ API contracts defined for both Rust and Python APIs. Error formats standardized via custom error types (OtlpError, OtlpConfigError, etc.). Configuration patterns consistent (YAML, env vars, programmatic API with OTLP_* prefix). Format conversion transparent to users (automatic based on forwarding config). Logging structured using tracing crate.

- **Performance Requirements (IV)**: ✅ SLOs defined: 1000+ messages/sec, p95 latency < 100ms. Performance targets measurable via integration tests and benchmarks. Format conversion overhead measured and optimized. Async I/O used throughout (tokio). Resource cleanup via RAII patterns. Performance regressions caught by CI/CD benchmarks.

- **Observability & Reliability (V)**: ✅ Structured logging via tracing crate with appropriate log levels. Format conversion operations logged for debugging. Metrics can be exposed via Prometheus-compatible endpoints (future enhancement). Health check endpoints planned for standalone service. Error rates and latency tracked. Graceful degradation for forwarding failures (doesn't block local storage). Format conversion errors handled gracefully.

**Post-Phase 1 Re-check**: All principles remain compliant. Dual protocol support and format conversion add complexity but are justified by specification requirements and provide backward compatibility and flexibility.

## Project Structure

### Documentation (this feature)

```text
specs/001-otlp-arrow-library/
├── plan.md              # This file (/speckit.plan command output)
├── research.md          # Phase 0 output (/speckit.plan command)
├── data-model.md        # Phase 1 output (/speckit.plan command)
├── quickstart.md        # Phase 1 output (/speckit.plan command)
├── contracts/           # Phase 1 output (/speckit.plan command)
│   ├── grpc-api.md      # gRPC API contracts (Protobuf and Arrow Flight)
│   ├── public-api.md    # Rust public API contract
│   └── python-api.md    # Python API contract
└── tasks.md             # Phase 2 output (/speckit.tasks command - NOT created by /speckit.plan)
```

### Source Code (repository root)

```text
src/
├── lib.rs               # Library root, public API re-exports
├── error.rs             # Custom error types
├── config/              # Configuration management
│   ├── mod.rs
│   ├── types.rs         # Configuration structs
│   └── loader.rs        # Configuration loading (YAML, env vars, programmatic)
├── otlp/                # OTLP processing
│   ├── mod.rs           # Module exports
│   ├── batch_writer.rs  # Batch buffering for writes
│   ├── exporter.rs      # Arrow IPC file exporter
│   ├── server.rs        # gRPC server (Protobuf)
│   ├── server_arrow.rs  # Arrow Flight server (OTAP)
│   ├── forwarder.rs     # Remote forwarding with format conversion
│   └── converter.rs    # Format conversion (Protobuf ↔ Arrow Flight)
├── api/                 # Public API
│   ├── mod.rs
│   └── public.rs        # OtlpLibrary struct and methods
├── python/              # Python bindings (PyO3)
│   ├── mod.rs
│   └── bindings.rs      # PyO3 bindings
├── mock/                # Mock service for testing
│   ├── mod.rs
│   └── service.rs       # MockOtlpService (supports both protocols)
└── bin/                 # Standalone service binary
    └── main.rs          # otlp-arrow-service binary

tests/
├── unit/                # Unit tests
│   ├── config/
│   ├── otlp/
│   │   ├── converter.rs # Format conversion unit tests
│   │   └── forwarder.rs # Forwarding unit tests
│   └── api/
├── integration/         # Integration tests
│   ├── test_grpc_protobuf_ingestion.rs
│   ├── test_grpc_arrow_flight_ingestion.rs
│   ├── test_public_api_traces.rs
│   ├── test_public_api_metrics.rs
│   ├── test_file_writing.rs
│   └── test_format_conversion.rs # Format conversion integration tests
└── contract/            # Contract tests
    └── test_otlp_protocol.rs
```

**Structure Decision**: Single Rust library project with modular organization. Source code organized by domain (config, otlp, api, mock, python). Format conversion logic isolated in dedicated `converter.rs` module within `otlp/` for maintainability. Test structure mirrors source organization with unit, integration, and contract test categories. Python bindings in separate `src/python/` module using PyO3. Standalone service binary in `src/bin/`.

## Complexity Tracking

> **Fill ONLY if Constitution Check has violations that must be justified**

| Violation | Why Needed | Simpler Alternative Rejected Because |
|-----------|------------|-------------------------------------|
| Dual gRPC server implementation | Specification requires both Protobuf and Arrow Flight protocols | Single protocol insufficient - breaks backward compatibility and doesn't meet spec requirements |
| Separate Arrow Flight server module | otel-arrow crate integration requires separate server implementation | Combining with Protobuf server would create tight coupling and violate separation of concerns |
| Format conversion module | Specification requires forwarding with format selection and automatic conversion | No conversion would limit forwarding flexibility - users must match input/output formats manually |
| Mock service dual protocol support | Testing requirements mandate validation of both integration paths | Single protocol mock service would not validate complete system behavior |
