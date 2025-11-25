# Implementation Plan: Python OpenTelemetry SDK Adapter Classes

**Branch**: `001-python-otel-adapters` | **Date**: 2025-11-25 | **Spec**: [spec.md](spec.md)
**Input**: Feature specification from `/specs/001-python-otel-adapters/spec.md`

**Note**: This template is filled in by the `/speckit.plan` command. See `.specify/templates/commands/plan.md` for the execution workflow.

## Summary

Provide built-in Python adapter classes that implement Python OpenTelemetry SDK exporter interfaces (`MetricExporter`, `SpanExporter`), enabling seamless integration between Python OpenTelemetry SDK and `OtlpLibrary` without requiring Python developers to write custom adapter code. The adapters will bridge between Python OpenTelemetry SDK types and the library's Python API methods, handle type conversion, error conversion, and lifecycle management.

## Technical Context

**Language/Version**: Rust 1.75+ (stable), Python 3.11+  
**Primary Dependencies**: PyO3 0.20, opentelemetry-python (Python OpenTelemetry SDK), opentelemetry-sdk 0.31 (Rust)  
**Storage**: N/A (adapters delegate to existing `OtlpLibrary` storage)  
**Testing**: pytest (Python), cargo test (Rust), Python OpenTelemetry SDK test utilities  
**Target Platform**: Windows, Linux, macOS (cross-platform Python bindings)  
**Project Type**: Library extension (Python bindings enhancement)  
**Performance Goals**: Type conversion overhead < 10% of export time, adapter creation < 1ms  
**Constraints**: Must preserve 100% of metric/span data during type conversion, support concurrent use from multiple Python OpenTelemetry SDK components  
**Scale/Scope**: Support standard Python OpenTelemetry SDK integration patterns, handle typical metric/span batch sizes (100-1000 items per batch)

## Constitution Check

*GATE: Must pass before Phase 0 research. Re-check after Phase 1 design.*

Verify compliance with OTLP Rust Service Constitution principles:

- **Code Quality (I)**: ✅ Design follows Rust best practices. Adapter implementations will be modular, well-documented, and follow SOLID principles. Type conversion logic will be isolated in dedicated modules for maintainability. Complexity will be kept low through clear separation of concerns (type conversion, error handling, lifecycle management).

- **Testing Standards (II)**: ✅ Testing strategy defined. TDD approach: tests for adapter interfaces → implementation → tests pass. Coverage targets: 85%+ for adapter code. Test types planned:
  - Unit tests: Adapter methods, type conversion functions, error handling
  - Integration tests: Full Python OpenTelemetry SDK integration (PeriodicExportingMetricReader, BatchSpanProcessor)
  - Contract tests: Verify Python OpenTelemetry SDK interface compliance
  - Performance tests: Type conversion overhead, concurrent usage scenarios
  - Python tests: pytest-based tests for adapter functionality

- **User Experience Consistency (III)**: ✅ API contracts will be consistent with existing Python API patterns. Error formats will follow Python OpenTelemetry SDK conventions. Configuration patterns will leverage existing `OtlpLibrary` configuration. Documentation will include examples for common integration patterns.

- **Performance Requirements (IV)**: ✅ SLOs defined. Performance targets:
  - Type conversion overhead: < 10% of total export time
  - Adapter creation: < 1ms
  - Concurrent export handling: No data corruption or race conditions
  - Memory usage: Bounded, no leaks during Python garbage collection

- **Observability & Reliability (V)**: ✅ Logging, metrics, tracing planned. Adapters will log errors during type conversion and export operations. Error context will be preserved for debugging. Health checks will verify adapter validity (library instance not shut down).

- **Commit Workflow**: ✅ Before committing, ensure CHANGELOG.md is updated, all docs are current, `cargo clippy` and `cargo fmt` pass, all tests pass (Rust and Python), and commits are GPG signed.

Any violations or exceptions MUST be documented in the Complexity Tracking section below.

## Project Structure

### Documentation (this feature)

```text
specs/001-python-otel-adapters/
├── plan.md              # This file (/speckit.plan command output)
├── research.md          # Phase 0 output (/speckit.plan command)
├── data-model.md        # Phase 1 output (/speckit.plan command)
├── quickstart.md        # Phase 1 output (/speckit.plan command)
├── contracts/           # Phase 1 output (/speckit.plan command)
│   ├── python-api.md    # Python adapter API contract
│   └── type-conversion.md # Type conversion contract
└── tasks.md             # Phase 2 output (/speckit.tasks command - NOT created by /speckit.plan)
```

### Source Code (repository root)

```text
src/
├── python/               # Python bindings (PyO3)
│   ├── mod.rs
│   ├── bindings.rs      # Existing PyO3 bindings for OtlpLibrary
│   └── adapters.rs      # NEW: Python OpenTelemetry SDK adapter implementations
│       ├── metric_adapter.rs  # MetricExporter adapter
│       ├── span_adapter.rs    # SpanExporter adapter
│       └── conversion.rs      # Type conversion utilities

tests/
├── python/              # Python tests
│   ├── test_adapters_metrics.py    # NEW: Metric adapter tests
│   ├── test_adapters_spans.py      # NEW: Span adapter tests
│   ├── test_type_conversion.py     # NEW: Type conversion tests
│   └── test_integration_otel_sdk.py # NEW: Full Python OpenTelemetry SDK integration
├── integration/        # Integration tests
│   └── test_python_otel_adapters.rs # NEW: Rust-side integration tests for adapters
└── unit/               # Unit tests
    └── python/         # NEW: Unit tests for adapter components
        └── test_adapters.rs
```

**Structure Decision**: Extend existing Python bindings module (`src/python/`) with new adapter implementations. Adapters will be implemented in Rust using PyO3, implementing Python OpenTelemetry SDK interfaces. Type conversion logic will be isolated in dedicated modules for maintainability and testability. Test structure mirrors source organization with Python tests for adapter functionality and Rust tests for underlying components.

## Complexity Tracking

> **Fill ONLY if Constitution Check has violations that must be justified**

| Violation | Why Needed | Simpler Alternative Rejected Because |
|-----------|------------|-------------------------------------|
| Python OpenTelemetry SDK type conversion layer | Python OpenTelemetry SDK uses different types than Rust SDK, requiring conversion between Python types and library-compatible formats | Direct type usage insufficient - Python OpenTelemetry SDK types are Python-specific and cannot be directly used with Rust library API |
| Dual adapter implementations (metrics + spans) | Specification requires both MetricExporter and SpanExporter adapters for complete observability integration | Single adapter insufficient - Python OpenTelemetry SDK has separate interfaces for metrics and traces, requiring separate implementations |
| Python garbage collection handling | Python adapters must remain valid while in use by Python OpenTelemetry SDK, requiring careful reference management | Simple wrapper insufficient - Python OpenTelemetry SDK may hold references to adapters across async boundaries, requiring proper lifetime management |

## Phase 0: Outline & Research

### Research Tasks

1. **Python OpenTelemetry SDK Exporter Interfaces**
   - Research Python OpenTelemetry SDK's `MetricExporter` interface requirements
   - Research Python OpenTelemetry SDK's `SpanExporter` interface requirements
   - Document required methods, signatures, and return types
   - Identify version compatibility requirements

2. **Type Conversion Patterns**
   - Research Python OpenTelemetry SDK metric data structures
   - Research Python OpenTelemetry SDK span data structures
   - Identify conversion patterns from Python types to library-compatible formats
   - Document data mapping between Python OpenTelemetry SDK and library types

3. **PyO3 Interface Implementation Patterns**
   - Research how to implement Python abstract base classes (ABC) from Rust
   - Research PyO3 patterns for implementing Python interfaces
   - Identify best practices for Python object lifecycle management
   - Document error conversion patterns (Rust errors → Python exceptions)

4. **Python OpenTelemetry SDK Integration Patterns**
   - Research `PeriodicExportingMetricReader` usage patterns
   - Research `BatchSpanProcessor` usage patterns
   - Identify common integration scenarios and edge cases
   - Document testing patterns for Python OpenTelemetry SDK integrations

5. **Cross-Platform Compatibility**
   - Research platform-specific considerations for Python bindings
   - Identify Windows/Linux/macOS differences in Python type handling
   - Document Python version compatibility (3.11+)

### Research Output

See `research.md` for consolidated findings.

## Phase 1: Design & Contracts

### Data Model

See `data-model.md` for entity definitions:
- Python Metric Exporter Adapter entity
- Python Span Exporter Adapter entity
- Type Conversion Layer entity

### API Contracts

See `contracts/python-api.md` for:
- `metric_exporter()` method contract
- `span_exporter()` method contract
- Adapter class interfaces
- Error handling contracts

See `contracts/type-conversion.md` for:
- Metric type conversion contract
- Span type conversion contract
- Error conversion contract

### Quickstart Guide

See `quickstart.md` for:
- Installation instructions
- Basic usage examples
- Integration with Python OpenTelemetry SDK
- Common patterns and best practices

## Constitution Check (Post-Design)

*Re-evaluated after Phase 1 design completion.*

Verify compliance with OTLP Rust Service Constitution principles:

- **Code Quality (I)**: ✅ Design maintains Rust best practices. Adapter implementations are modular with clear separation of concerns (type conversion, error handling, lifecycle management). Type conversion logic is isolated in dedicated modules. Complexity is kept low through delegation patterns and clear interfaces.

- **Testing Standards (II)**: ✅ Testing strategy fully defined. TDD approach with comprehensive test coverage:
  - Unit tests: Adapter methods, type conversion functions, error handling (Rust and Python)
  - Integration tests: Full Python OpenTelemetry SDK integration (PeriodicExportingMetricReader, BatchSpanProcessor)
  - Contract tests: Verify Python OpenTelemetry SDK interface compliance
  - Performance tests: Type conversion overhead, concurrent usage scenarios
  - Python tests: pytest-based tests for adapter functionality
  - Coverage target: 85%+ for all adapter code

- **User Experience Consistency (III)**: ✅ API contracts are consistent with existing Python API patterns. Error formats follow Python OpenTelemetry SDK conventions. Configuration patterns leverage existing `OtlpLibrary` configuration. Documentation includes comprehensive examples and quickstart guide.

- **Performance Requirements (IV)**: ✅ SLOs clearly defined and measurable:
  - Type conversion overhead: < 10% of total export time
  - Adapter creation: < 1ms
  - Concurrent export handling: No data corruption or race conditions
  - Memory usage: Bounded, no leaks during Python garbage collection
  - All targets are measurable and achievable

- **Observability & Reliability (V)**: ✅ Logging, metrics, tracing planned. Adapters will log errors during type conversion and export operations. Error context will be preserved for debugging. Health checks will verify adapter validity. Error messages include original error details for observability.

- **Commit Workflow**: ✅ All commit workflow requirements will be followed:
  - CHANGELOG.md will be updated with adapter feature
  - Documentation will be updated (quickstart, API contracts, data model)
  - `cargo clippy` and `cargo fmt` will pass
  - All tests will pass (Rust and Python)
  - Commits will be GPG signed

**No violations identified. Design complies with all constitution principles.**

## Summary

The implementation plan for Python OpenTelemetry SDK adapter classes is complete. The plan includes:

1. **Research Phase (Phase 0)**: All research questions resolved, documented in `research.md`
2. **Design Phase (Phase 1)**: Data model, API contracts, type conversion contracts, and quickstart guide created
3. **Constitution Compliance**: All principles verified and compliant
4. **Ready for Implementation**: Plan is ready for `/speckit.tasks` to break down into implementation tasks

**Key Deliverables**:
- `research.md`: Research findings and decisions
- `data-model.md`: Entity definitions and relationships
- `contracts/python-api.md`: Python API contract
- `contracts/type-conversion.md`: Type conversion contract
- `quickstart.md`: Usage guide and examples

**Next Steps**:
- Run `/speckit.tasks` to create implementation task breakdown
- Begin implementation following TDD approach
- Follow constitution principles throughout implementation
