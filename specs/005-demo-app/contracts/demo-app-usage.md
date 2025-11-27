# Demo Application Usage Contract

**Feature**: 005-demo-app  
**Date**: 2025-11-26

## Overview

This document defines the usage contract for the demo Rust application. The demo serves as both a verification tool and a reference implementation for developers using the OTLP SDK.

## Execution Contract

### Command Line Interface

**Command**: `cargo run --example demo-app`

**Behavior**:
- Compiles and runs the demo application
- Enables dashboard by default
- Generates mock metrics and spans
- Runs continuously until interrupted (Ctrl+C) or completes single execution

**Exit Codes**:
- `0`: Success
- `1`: Error during execution (configuration error, initialization failure, etc.)

**Output**:
- Prints dashboard URL to stdout (e.g., "Dashboard available at http://127.0.0.1:8080")
- Prints status messages for data generation
- Prints error messages to stderr on failure

### Configuration Contract

**Default Configuration**:
- Dashboard: Enabled
- Dashboard Port: 8080
- Output Directory: `./output_dir`
- Write Interval: 5 seconds
- Static Directory: `./dashboard/dist`

**Configuration Override**:
- Can be modified in code (demonstrates `ConfigBuilder` usage)
- Environment variables can override (OTLP_* prefix)
- YAML config file can be used (if library supports it)

### Data Generation Contract

**Metrics Generation**:
- Generates at least 10 distinct metrics during execution
- Uses `ResourceMetrics::default()` (SDK limitation)
- Exports via `library.export_metrics()` or `library.export_metrics_ref()`
- Metrics written to `{output_dir}/otlp/metrics/` in Arrow IPC format

**Spans Generation**:
- Generates at least 10 distinct spans during execution
- Creates spans with different kinds (Server, Client, Internal)
- Creates parent-child relationships (demonstrates trace structure)
- Includes realistic attributes (service.name, http.method, http.status_code, etc.)
- Spans written to `{output_dir}/otlp/traces/` in Arrow IPC format

**Generation Pattern**:
- Continuous mode: Generates data every 2-5 seconds until interrupted
- Single execution mode: Generates batch of data, then exits
- Data values change over time to demonstrate time-series visualization

### Dashboard Contract

**Availability**:
- Dashboard starts automatically when demo runs (if enabled)
- Accessible at `http://127.0.0.1:8080` (or configured port)
- Dashboard reads Arrow IPC files from output directory
- Dashboard updates automatically as new data arrives

**Display Requirements**:
- Metrics visible in dashboard within 10 seconds of generation (SC-001)
- Spans visible in dashboard within 10 seconds of generation (SC-001)
- Time-series patterns visible for metrics
- Trace relationships visible for spans

### Error Handling Contract

**Configuration Errors**:
- Invalid dashboard port: Print error, exit with code 1
- Missing static directory: Print error, exit with code 1
- Invalid output directory: Print error, exit with code 1

**Runtime Errors**:
- Export failures: Log warning, continue execution
- Dashboard startup failure: Log error, continue without dashboard
- File write failures: Log error, continue execution

**Graceful Shutdown**:
- On Ctrl+C: Flush pending data, shutdown library, exit cleanly
- On error: Attempt graceful shutdown, exit with code 1

## Code Structure Contract

### Required Sections

The demo application code MUST include these sections with clear comments:

1. **Initialization**: Library setup with dashboard configuration
2. **Metric Creation**: Examples of creating and exporting metrics
3. **Span Creation**: Examples of creating spans with different kinds and relationships
4. **Batch Export**: Examples of exporting multiple items
5. **Continuous Generation**: Pattern for generating data over time (if applicable)
6. **Graceful Shutdown**: Flush and shutdown pattern

### Documentation Requirements

- **Doc Comments**: Module-level and function-level documentation
- **Inline Comments**: Explain purpose of each SDK method call (80% coverage per SC-004)
- **Example Comments**: Show expected behavior and usage patterns

### Code Quality Requirements

- Must pass `cargo clippy` with no warnings
- Must pass `cargo fmt`
- Must follow Rust best practices
- Must be readable by developers with intermediate Rust knowledge

## Reference Implementation Contract

The demo application MUST demonstrate:

1. **Dashboard Configuration**: How to enable dashboard using `ConfigBuilder`
2. **Library Initialization**: How to create `OtlpLibrary` instance
3. **Metric Export**: How to export metrics (individual and batch)
4. **Span Export**: How to export spans (individual and batch)
5. **Span Relationships**: How to create parent-child span relationships
6. **Span Attributes**: How to add attributes to spans
7. **Span Kinds**: How to use different span kinds
8. **Graceful Shutdown**: How to flush and shutdown properly

## Testing Contract

**Unit Tests**:
- Verify demo compiles successfully
- Verify demo runs without errors
- Verify configuration is correct

**Integration Tests**:
- Verify dashboard starts and is accessible
- Verify data appears in dashboard
- Verify Arrow IPC files are created
- Verify graceful shutdown works

## Compatibility Contract

**Platform Support**:
- Windows: Must compile and run
- Linux: Must compile and run
- macOS: Must compile and run

**Rust Version**:
- Minimum: Rust 1.75+ (stable channel)
- Edition: 2021

**Dependencies**:
- No additional dependencies beyond OTLP library
- Uses only standard library and library's public API

