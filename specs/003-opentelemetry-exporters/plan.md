# Implementation Plan: Built-in OpenTelemetry Exporter Implementations

**Branch**: `003-opentelemetry-exporters` | **Date**: 2025-01-27 | **Spec**: [spec.md](./spec.md)
**Input**: Feature specification from `/specs/003-opentelemetry-exporters/spec.md`

## Summary

This feature adds built-in OpenTelemetry SDK exporter implementations (`PushMetricExporter` and `SpanExporter`) that wrap `OtlpLibrary`, eliminating the need for consumers to create custom wrapper types. The implementation includes:

1. **Reference-based export method** (`export_metrics_ref`): Accepts `&ResourceMetrics` instead of owned `ResourceMetrics` for efficient integration with OpenTelemetry SDK's periodic readers
2. **Built-in exporter types**: `OtlpMetricExporter` and `OtlpSpanExporter` that implement OpenTelemetry SDK traits and delegate to `OtlpLibrary` methods
3. **Convenience methods**: `metric_exporter()` and `span_exporter()` on `OtlpLibrary` that return ready-to-use exporter instances
4. **Python bindings**: Expose exporter creation methods and types through PyO3 bindings for Python integration

The technical approach leverages existing `OtlpLibrary` infrastructure, adds reference-based export to avoid unnecessary cloning, and implements OpenTelemetry SDK trait interfaces with proper error conversion and lifecycle management.

## Technical Context

**Language/Version**: Rust 1.75+ (stable channel, latest stable)  
**Primary Dependencies**: 
- `opentelemetry-sdk` 0.31 (for `PushMetricExporter` and `SpanExporter` traits)
- `opentelemetry` 0.31 (for OpenTelemetry types)
- `pyo3` 0.20 (for Python bindings)
- `tokio` 1.35 (for async runtime)
- `futures` 0.3 (for `BoxFuture` and async utilities)

**Storage**: N/A (exporters delegate to existing `OtlpLibrary` file storage)  
**Testing**: 
- `cargo test` for unit and integration tests
- OpenTelemetry SDK integration tests with `PeriodicReader` and `TracerProvider`
- Python bindings tests with Python OpenTelemetry SDK
- Cross-platform testing on Windows, Linux, macOS

**Target Platform**: Cross-platform (Windows, Linux, macOS)  
**Project Type**: Library (embedded library with public API)  
**Performance Goals**: 
- Reference-based export reduces memory allocations by 50%+ compared to owned export
- Exporter operations should not add significant latency (<10ms overhead per export)
- Support concurrent use from multiple OpenTelemetry SDK components

**Constraints**: 
- Must maintain backward compatibility with existing `OtlpLibrary` API
- Must support OpenTelemetry SDK 0.31 (current dependency version)
- Python bindings must support Python 3.11+
- Must function identically on all target platforms

**Scale/Scope**: 
- Library-level feature (adds new types and methods to existing library)
- No new external dependencies required
- Extends existing `OtlpLibrary` with ~200-300 lines of new code
- Python bindings extension (~100-150 lines)

## Constitution Check

*GATE: Must pass before Phase 0 research. Re-check after Phase 1 design.*

Verify compliance with OTLP Rust Service Constitution principles:

- **Code Quality (I)**: ✅ Design follows Rust best practices:
  - New types implement `Debug` and `Clone` where appropriate
  - Error handling uses `Result<T, E>` (no panics in library code)
  - Functions documented with doc comments
  - Complexity kept low (simple delegation pattern)
  - Public APIs have comprehensive documentation with examples

- **Testing Standards (II)**: ✅ Testing strategy defined:
  - TDD approach: Write tests for `export_metrics_ref` first, then implementation
  - Unit tests for exporter implementations (error conversion, lifecycle)
  - Integration tests with OpenTelemetry SDK (`PeriodicReader`, `TracerProvider`)
  - Python bindings tests with Python OpenTelemetry SDK
  - Cross-platform tests on Windows, Linux, macOS
  - Target: 85%+ code coverage for new code

- **User Experience Consistency (III)**: ✅ API contracts consistent:
  - Exporter methods follow same patterns as existing `OtlpLibrary` methods
  - Error types converted appropriately but maintain context
  - Configuration patterns consistent (exporters use library's config)
  - Documentation follows existing library documentation style

- **Performance Requirements (IV)**: ✅ SLOs defined:
  - Reference-based export: 50%+ reduction in memory allocations
  - Exporter overhead: <10ms per export operation
  - Concurrent use: No data corruption or race conditions
  - Measurable via benchmarks and integration tests

- **Observability & Reliability (V)**: ✅ Observability planned:
  - Errors logged via existing `tracing` infrastructure
  - Export failures logged with context
  - No new metrics required (delegates to existing library metrics)
  - Health checks not affected (library-level feature)

- **Commit Workflow**: ✅ Will comply:
  - CHANGELOG.md updated with new methods and types
  - Documentation updated (API docs, examples)
  - `cargo fmt` and `cargo clippy` pass
  - All tests pass
  - Commits GPG signed

**No violations identified.** Design is straightforward and follows existing patterns.

## Project Structure

### Documentation (this feature)

```text
specs/003-opentelemetry-exporters/
├── plan.md              # This file (/speckit.plan command output)
├── research.md          # Phase 0 output (/speckit.plan command)
├── data-model.md        # Phase 1 output (/speckit.plan command)
├── quickstart.md        # Phase 1 output (/speckit.plan command)
├── contracts/           # Phase 1 output (/speckit.plan command)
│   ├── rust-api.md      # Rust API contract for exporters
│   └── python-api.md    # Python API contract for exporters
└── tasks.md             # Phase 2 output (/speckit.tasks command - NOT created by /speckit.plan)
```

### Source Code (repository root)

```text
src/
├── api/
│   └── public.rs        # OtlpLibrary - add metric_exporter() and span_exporter() methods
├── otlp/
│   ├── exporter.rs       # Add OtlpMetricExporter and OtlpSpanExporter types
│   └── mod.rs           # Export new exporter types
├── python/
│   └── bindings.rs      # Add Python bindings for exporter methods and types
└── lib.rs                # Re-export new exporter types

tests/
├── integration/
│   ├── test_exporters_opentelemetry_sdk.rs  # Integration with OpenTelemetry SDK
│   └── test_exporters_python.rs             # Python bindings integration tests
└── unit/
    └── otlp/
        ├── test_metric_exporter.rs          # Unit tests for OtlpMetricExporter
        └── test_span_exporter.rs             # Unit tests for OtlpSpanExporter
```

**Structure Decision**: This is a library extension feature. New code is added to existing modules:
- Exporter types added to `src/otlp/exporter.rs` (alongside existing `FileMetricExporter` and `FileSpanExporter`)
- Convenience methods added to `src/api/public.rs` (on `OtlpLibrary`)
- Python bindings extended in `src/python/bindings.rs`
- Tests follow existing test structure patterns

## Complexity Tracking

> **No violations identified** - Design is straightforward and follows existing patterns.

## Phase 0: Research - COMPLETE

**Status**: ✅ All research questions resolved

**Output**: `research.md` - Documents technical decisions for:
- ResourceMetrics reference-based export approach
- OpenTelemetry SDK trait implementation patterns
- Python bindings strategy
- Error conversion strategy
- Lifecycle management approach
- Concurrent use support

**Key Decisions**:
- Use reference-based export (`export_metrics_ref`) to avoid cloning
- Implement traits on new types wrapping `Arc<OtlpLibrary>`
- Extend Python bindings with exporter creation methods
- Convert errors to `OTelSdkError::InternalFailure` with context
- Exporters handle shutdown gracefully but don't shut down library

## Phase 1: Design & Contracts - COMPLETE

**Status**: ✅ All design artifacts generated

**Outputs**:
- `data-model.md` - Entity definitions for exporters and extended library
- `contracts/rust-api.md` - Rust API contract for exporter methods and types
- `contracts/python-api.md` - Python API contract for exporter methods and types
- `quickstart.md` - Quickstart guide with examples

**Agent Context**: ✅ Updated via `.specify/scripts/bash/update-agent-context.sh`

## Constitution Check (Post-Design)

*Re-checked after Phase 1 design completion*

- **Code Quality (I)**: ✅ Design follows Rust best practices, documented, low complexity
- **Testing Standards (II)**: ✅ Testing strategy defined (TDD, unit/integration/Python tests, 85%+ coverage)
- **User Experience Consistency (III)**: ✅ API contracts consistent, error handling documented
- **Performance Requirements (IV)**: ✅ SLOs defined (50%+ memory reduction, <10ms overhead)
- **Observability & Reliability (V)**: ✅ Errors logged, no new metrics needed
- **Commit Workflow**: ✅ Will comply (CHANGELOG, docs, tests, GPG signing)

**No violations identified.** Design is ready for implementation.

## Next Steps

Ready for `/speckit.tasks` to break down implementation into concrete tasks.
