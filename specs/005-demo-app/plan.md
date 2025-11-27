# Implementation Plan: Demo Rust Application

**Branch**: `005-demo-app` | **Date**: 2025-11-26 | **Spec**: [spec.md](./spec.md)
**Input**: Feature specification from `/specs/005-demo-app/spec.md`

**Note**: This template is filled in by the `/speckit.plan` command. See `.specify/templates/commands/plan.md` for the execution workflow.

## Summary

Create a demo Rust application (`examples/demo-app.rs`) that demonstrates OTLP SDK usage by enabling the dashboard, generating mock metrics and spans, and serving as a reference implementation for developers. The demo will showcase all primary SDK patterns: initialization with dashboard enabled, metric creation, span creation (with different kinds and relationships), batch export, and graceful shutdown.

## Technical Context

**Language/Version**: Rust 1.75+ (stable channel, edition 2021)  
**Primary Dependencies**: 
- `otlp_arrow_library` (this library)
- `opentelemetry` 0.31
- `opentelemetry_sdk` 0.31
- `tokio` 1.35 (async runtime)
- `tracing` (logging)

**Storage**: Local filesystem (Arrow IPC format files in output directory)  
**Testing**: `cargo test` with unit tests for demo app functionality  
**Target Platform**: Cross-platform (Windows, Linux, macOS)  
**Project Type**: Single example application (standalone executable)  
**Performance Goals**: Demo generates data at reasonable rate (1-10 metrics/spans per second) for visualization purposes  
**Constraints**: 
- Must compile and run without additional dependencies beyond OTLP library
- Must work with dashboard static files in default location (`./dashboard/dist`)
- Must demonstrate realistic but simple mock data patterns
- ResourceMetrics construction limited by private fields in opentelemetry-sdk 0.31

**Scale/Scope**: 
- Single example file (~200-300 lines)
- Generates 10+ distinct metrics and 10+ distinct spans
- Runs continuously or as single execution
- Serves as reference for developers integrating SDK

## Constitution Check

*GATE: Must pass before Phase 0 research. Re-check after Phase 1 design.*

Verify compliance with OTLP Rust Service Constitution principles:

- **Code Quality (I)**: ✅ Design follows Rust best practices. Example code will be well-documented with doc comments and inline comments. Functions will be focused and maintainable. Code will pass `cargo clippy` and `cargo fmt`.

- **Testing Standards (II)**: ✅ Testing strategy defined. Unit tests will verify demo app compiles and runs successfully. Integration tests will verify dashboard displays generated data. Test coverage will validate all SDK usage patterns demonstrated.

- **User Experience Consistency (III)**: ✅ API usage follows consistent patterns from existing examples (`embedded.rs`, `standalone.rs`). Configuration uses `ConfigBuilder` pattern. Error handling demonstrates best practices.

- **Performance Requirements (IV)**: ✅ Performance targets are appropriate for demo (not performance-critical). Demo generates data at reasonable rate for visualization. No specific SLOs required for demo application.

- **Observability & Reliability (V)**: ✅ Demo uses `tracing` for structured logging. Demonstrates graceful shutdown with flush. Error handling shows proper error propagation.

- **Commit Workflow**: ✅ Before committing, CHANGELOG.md will be updated, all docs will be current, `cargo clippy` and `cargo fmt` will pass, all tests will pass, and commits will be GPG signed.

**No violations identified.** Demo application is a simple example that follows all constitution principles.

## Project Structure

### Documentation (this feature)

```text
specs/005-demo-app/
├── plan.md              # This file (/speckit.plan command output)
├── research.md          # Phase 0 output (/speckit.plan command)
├── data-model.md        # Phase 1 output (/speckit.plan command)
├── quickstart.md        # Phase 1 output (/speckit.plan command)
├── contracts/           # Phase 1 output (/speckit.plan command)
└── tasks.md             # Phase 2 output (/speckit.tasks command - NOT created by /speckit.plan)
```

### Source Code (repository root)

```text
examples/
├── demo-app.rs          # New demo application (this feature)
├── embedded.rs          # Existing embedded example
├── standalone.rs        # Existing standalone example
└── python_example.py    # Existing Python example

tests/
├── integration/
│   └── test_demo_app.rs # Integration test for demo app
└── unit/
    └── examples/
        └── test_demo_app.rs # Unit tests for demo app
```

**Structure Decision**: Single example file in `examples/` directory following existing project conventions. The demo app will be a standalone executable runnable via `cargo run --example demo-app`. Tests will be added to verify the demo compiles, runs, and generates expected data.

## Complexity Tracking

> **No violations identified** - Demo application is straightforward and follows existing patterns.
