# Implementation Plan: OTLP Arrow Flight Library

**Branch**: `001-otlp-arrow-library` | **Date**: 2024-12-19 | **Spec**: [spec.md](./spec.md)
**Input**: Feature specification from `/specs/001-otlp-arrow-library/spec.md`

**Note**: This template is filled in by the `/speckit.plan` command. See `.specify/templates/commands/plan.md` for the execution workflow.

## Summary

Build a cross-platform Rust library that receives OTLP (OpenTelemetry Protocol) messages via both gRPC with Protobuf (standard OTLP) and gRPC with Arrow Flight IPC (OTAP), writes them to local files in Arrow IPC Streaming format, and optionally forwards them to remote endpoints with automatic format conversion. The library must support both standalone service mode and embedded library usage with public API methods callable from both Rust and Python projects. Based on existing OTLP implementation patterns from cap-gl-consumer-rust codebase, with Arrow Flight support via otel-arrow Rust crate and format conversion for flexible forwarding.

## Technical Context

**Language/Version**: Rust stable (latest), Python 3.11+  
**Primary Dependencies**: 
- `opentelemetry` (0.31), `opentelemetry-sdk` (0.31), `opentelemetry-otlp` (0.31), `opentelemetry-proto` (0.31) - OpenTelemetry SDK and OTLP support
- `arrow` (57), `arrow-array` (57), `arrow-flight` (57) - Apache Arrow IPC and Flight IPC
- `tokio` (1.35+) - Async runtime
- `tonic` (0.14) - gRPC framework
- `serde` (1.0), `serde_yaml` (0.9) - Serialization/configuration
- `pyo3` (0.20) - Python bindings (FFI for Python support)
- `anyhow` (1.0), `thiserror` (1.0) - Error handling
- `tracing` (0.1) - Structured logging

**Storage**: Local filesystem (Arrow IPC Streaming format files)  
**Testing**: `cargo test` with unit, integration, and contract tests. Mock service for end-to-end testing supporting both gRPC protocols. Python tests using pytest for Python bindings.  
**Target Platform**: Cross-platform (Windows, Linux, macOS)  
**Project Type**: Rust library (can be used standalone or embedded) with Python bindings  
**Performance Goals**: 
- At least 1000 OTLP messages per second without data loss
- p95 latency < 100ms (message receipt to batch write initiation)
- Batch writes every 5 seconds (default, configurable)

**Constraints**: 
- Minimum Python version: 3.11
- Minimum code coverage: 85% per file
- All code must pass `cargo clippy` with no warnings
- TDD mandatory for all new features

**Scale/Scope**: 
- Single Rust library project with modular organization
- Source code organized by domain (config, otlp, api, mock, python)
- Test structure mirrors source organization
- Python bindings in separate `src/python/` module using PyO3
- Standalone service binary in `src/bin/`

## Constitution Check

*GATE: Must pass before Phase 0 research. Re-check after Phase 1 design.*

Verify compliance with OTLP Rust Service Constitution principles:

- **Code Quality (I)**: ✅ Design follows Rust best practices with modular organization, comprehensive documentation, and SOLID principles. Complexity managed through domain separation (config, otlp, api, mock, python modules).
- **Testing Standards (II)**: ✅ Testing strategy defined with TDD mandatory. Coverage targets: 85% minimum per file. Test types: unit (fast, isolated), integration (external interfaces), contract (OTLP protocol compliance), performance (benchmarks). Mock service enables end-to-end testing without external dependencies.
- **User Experience Consistency (III)**: ✅ API contracts defined for both Rust and Python APIs. Error formats standardized via custom error types (OtlpError, OtlpConfigError, etc.). Configuration patterns consistent (YAML, env vars, programmatic API with OTLP_* prefix). Format conversion transparent to users (automatic based on forwarding config). Logging structured using tracing crate.
- **Performance Requirements (IV)**: ✅ SLOs defined: 1000 msg/s throughput, p95 latency < 100ms, batch writes every 5s. Performance targets measurable via benchmarks. Async operations for I/O-bound tasks. Resource cleanup via RAII patterns.
- **Observability & Reliability (V)**: ✅ Structured logging via tracing crate. Metrics collection planned for critical operations. Health check endpoints for standalone service. Error handling graceful without message loss. Circuit breakers for remote forwarding.

Any violations or exceptions MUST be documented in the Complexity Tracking section below.

## Project Structure

### Documentation (this feature)

```text
specs/001-otlp-arrow-library/
├── plan.md              # This file (/speckit.plan command output)
├── research.md          # Phase 0 output (/speckit.plan command)
├── data-model.md        # Phase 1 output (/speckit.plan command)
├── quickstart.md        # Phase 1 output (/speckit.plan command)
├── contracts/           # Phase 1 output (/speckit.plan command)
│   ├── grpc-api.md      # gRPC API contract
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
│   ├── types.rs         # Config, ForwardingConfig, AuthConfig, ProtocolConfig
│   └── loader.rs        # YAML, env vars, programmatic loading
├── otlp/                # OTLP protocol handling
│   ├── mod.rs
│   ├── server.rs        # gRPC Protobuf server
│   ├── server_arrow.rs  # gRPC Arrow Flight server
│   ├── exporter.rs      # File-based exporter (Arrow IPC)
│   ├── batch_writer.rs  # Batch writing logic
│   ├── converter.rs     # Format conversion (Protobuf ↔ Arrow Flight)
│   └── forwarder.rs     # Remote forwarding with format conversion
├── api/                 # Public API
│   ├── mod.rs
│   └── public.rs        # OtlpLibrary struct and public methods
├── mock/                 # Mock service for testing
│   ├── mod.rs
│   └── service.rs       # In-memory mock OTLP service
├── python/               # Python bindings (PyO3)
│   ├── mod.rs
│   └── bindings.rs      # PyO3 bindings for OtlpLibrary
└── bin/                  # Standalone service binary
    └── main.rs           # Main entry point for standalone mode

tests/
├── unit/                 # Unit tests (fast, isolated)
│   ├── config/
│   ├── otlp/
│   ├── api/
│   └── mock/
├── integration/         # Integration tests (external interfaces)
│   ├── test_grpc_protobuf_*.rs
│   ├── test_grpc_arrow_flight_*.rs
│   ├── test_forwarding_*.rs
│   ├── test_mock_service_*.rs
│   └── test_public_api_*.rs
├── contract/            # Contract tests (OTLP protocol compliance)
│   └── test_otlp_protocol.rs
├── bench/               # Performance benchmarks
│   ├── bench_throughput.rs
│   ├── bench_latency.rs
│   └── bench_format_conversion.rs
├── python/              # Python tests
│   ├── test_library_init.py
│   ├── test_trace_export.py
│   ├── test_metrics_export.py
│   └── test_integration.py
└── test_*.rs            # Root-level unit tests

examples/
├── standalone.rs        # Standalone service example
├── embedded.rs          # Embedded library example
└── python_example.py    # Python usage example
```

**Structure Decision**: Single Rust library project with modular organization. Source code organized by domain (config, otlp, api, mock, python). Format conversion logic isolated in dedicated `converter.rs` module within `otlp/` for maintainability. Test structure mirrors source organization with unit, integration, and contract test categories. Python bindings in separate `src/python/` module using PyO3. Standalone service binary in `src/bin/`.

## Complexity Tracking

> **Fill ONLY if Constitution Check has violations that must be justified**

| Violation | Why Needed | Simpler Alternative Rejected Because |
|-----------|------------|-------------------------------------|
| Dual protocol support (Protobuf + Arrow Flight) | Specification requires both protocols simultaneously | Single protocol insufficient - need backward compatibility (Protobuf) and performance (Arrow Flight) |
| Format conversion (Protobuf ↔ Arrow Flight) | Forwarding must support configurable output format regardless of input | Single format forwarding insufficient - users need flexibility to forward to different endpoint types |
| Python bindings (PyO3) | Specification requires public API callable from Python projects | Rust-only API insufficient - specification explicitly requires Python support |
| Mock service | End-to-end testing requires validation of both gRPC and public API paths | External OTLP service dependencies insufficient - need reliable, repeatable testing without network dependencies |
