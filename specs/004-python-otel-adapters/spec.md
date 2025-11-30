# Feature Specification: Python OpenTelemetry SDK Adapter Classes

**Feature Branch**: `004-python-otel-adapters`  
**Created**: 2025-11-25  
**Status**: Draft  
**Input**: User description: "Review the following issue and its comments - [Issue #6](https://github.com/pixie79/otlp-rust-service/issues/6)"

## User Scenarios & Testing *(mandatory)*

### User Story 1 - Direct Integration with Python OpenTelemetry SDK Metrics (Priority: P1)

A Python developer wants to integrate the library with Python OpenTelemetry SDK's metric collection system without writing custom adapter code. The system must provide a Python class that implements Python OpenTelemetry SDK's `MetricExporter` interface, allowing direct use with `PeriodicExportingMetricReader` and eliminating the need for boilerplate wrapper implementations.

**Why this priority**: This delivers the primary value proposition for Python developers - seamless integration with Python OpenTelemetry SDK's metric system. Without this, Python developers must create their own adapter classes, leading to repetitive code, inconsistent implementations, and maintenance burden. This story enables the complete integration experience that makes the library production-ready for Python OpenTelemetry SDK metric integration.

**Independent Test**: Can be fully tested by creating a metric exporter adapter from an `OtlpLibrary` instance and using it directly with Python OpenTelemetry SDK's `PeriodicExportingMetricReader` without any custom wrapper code. The test delivers a complete, working integration that demonstrates the library's ease of use for Python metric collection.

**Acceptance Scenarios**:

1. **Given** a Python `OtlpLibrary` instance is created, **When** a developer calls `metric_exporter_adapter()` method, **Then** the method returns a Python class that implements Python OpenTelemetry SDK's `MetricExporter` interface
2. **Given** a metric exporter adapter is created from the library, **When** it is used with Python OpenTelemetry SDK's `PeriodicExportingMetricReader`, **Then** metrics are automatically exported to the library's storage system at the configured intervals
3. **Given** a metric exporter adapter is used with Python OpenTelemetry SDK, **When** Python OpenTelemetry SDK calls the `export` method with Python metric data, **Then** the adapter converts Python metric types to library-compatible formats and exports them correctly
4. **Given** a metric exporter adapter is used with Python OpenTelemetry SDK, **When** errors occur during export, **Then** the adapter converts library errors to appropriate Python OpenTelemetry SDK error types while preserving error context
5. **Given** a metric exporter adapter is used with Python OpenTelemetry SDK, **When** Python OpenTelemetry SDK calls shutdown or flush methods, **Then** the adapter handles these lifecycle methods gracefully without affecting the library's lifecycle

---

### User Story 2 - Direct Integration with Python OpenTelemetry SDK Traces (Priority: P2)

A Python developer wants to integrate the library with Python OpenTelemetry SDK's trace collection system without writing custom adapter code. The system must provide a Python class that implements Python OpenTelemetry SDK's `SpanExporter` interface, allowing direct use with `BatchSpanProcessor` and `TracerProvider`, eliminating the need for boilerplate wrapper implementations.

**Why this priority**: This extends the convenience of adapter classes to trace collection, ensuring feature parity between metrics and traces. While Story 1 provides metric integration, this story enables complete observability integration for Python developers. This completes the Python OpenTelemetry SDK integration experience.

**Independent Test**: Can be fully tested by creating a span exporter adapter from an `OtlpLibrary` instance and using it directly with Python OpenTelemetry SDK's `BatchSpanProcessor` and `TracerProvider` without any custom wrapper code. The test delivers a complete, working integration that demonstrates the library's ease of use for Python trace collection.

**Acceptance Scenarios**:

1. **Given** a Python `OtlpLibrary` instance is created, **When** a developer calls `span_exporter_adapter()` method, **Then** the method returns a Python class that implements Python OpenTelemetry SDK's `SpanExporter` interface
2. **Given** a span exporter adapter is created from the library, **When** it is used with Python OpenTelemetry SDK's `BatchSpanProcessor` and `TracerProvider`, **Then** spans are automatically exported to the library's storage system when batches are ready
3. **Given** a span exporter adapter is used with Python OpenTelemetry SDK, **When** Python OpenTelemetry SDK calls the `export` method with Python span data, **Then** the adapter converts Python span types to library-compatible formats and exports them correctly
4. **Given** a span exporter adapter is used with Python OpenTelemetry SDK, **When** errors occur during export, **Then** the adapter converts library errors to appropriate Python OpenTelemetry SDK error types while preserving error context
5. **Given** a span exporter adapter is used with Python OpenTelemetry SDK, **When** Python OpenTelemetry SDK calls shutdown or flush methods, **Then** the adapter handles these lifecycle methods gracefully without affecting the library's lifecycle

---

### User Story 3 - Cross-Platform Compatibility and Python Version Support (Priority: P3)

A Python developer wants to use the adapter classes across different operating systems and Python versions without encountering platform-specific or version-specific issues. The system must ensure that adapter implementations function correctly on Windows, Linux, and macOS, and support Python 3.11 or higher.

**Why this priority**: This ensures broad accessibility and reliability of the adapter classes across different development and deployment environments. While Stories 1 and 2 provide core functionality, this story ensures that the adapters work consistently regardless of the developer's platform or Python version, making the library truly production-ready for diverse Python environments.

**Independent Test**: Can be fully tested by creating and using adapter classes on different operating systems (Windows, Linux, macOS) and different Python versions (3.11+), verifying that they function correctly and produce identical behavior across all tested environments. The test delivers confidence that the adapters work reliably in diverse deployment scenarios.

**Acceptance Scenarios**:

1. **Given** adapter classes are created on Windows, **When** they are used with Python OpenTelemetry SDK, **Then** they function correctly and export metrics and spans without platform-specific errors
2. **Given** adapter classes are created on Linux, **When** they are used with Python OpenTelemetry SDK, **Then** they function correctly and export metrics and spans without platform-specific errors
3. **Given** adapter classes are created on macOS, **When** they are used with Python OpenTelemetry SDK, **Then** they function correctly and export metrics and spans without platform-specific errors
4. **Given** adapter classes are created with Python 3.11, **When** they are used with Python OpenTelemetry SDK, **Then** they function correctly and export metrics and spans without version-specific errors
5. **Given** adapter classes are created with Python 3.12 or higher, **When** they are used with Python OpenTelemetry SDK, **Then** they function correctly and export metrics and spans without version-specific errors

---

### Edge Cases

- What happens when adapter classes are created from a library instance that has been shut down?
- How does the system handle errors when adapters attempt to export data after the library is shut down?
- What happens when Python OpenTelemetry SDK calls adapter methods with invalid or malformed data?
- How does the system handle memory pressure when exporting large batches of metrics or spans?
- What happens when Python OpenTelemetry SDK calls export methods concurrently from multiple threads?
- How does the system handle shutdown timeouts when Python OpenTelemetry SDK requests graceful shutdown?
- How does the system handle Python garbage collection of adapter objects while they're still in use by Python OpenTelemetry SDK?
- What happens when adapter classes are called from different Python threads or async contexts?
- How does the system handle platform-specific differences in type conversion between Windows, Linux, and macOS?
- What happens when type conversion fails between Python OpenTelemetry SDK types and library-compatible formats?
- How does the system handle differences in Python OpenTelemetry SDK versions that may have different interface requirements?

## Requirements *(mandatory)*

### Functional Requirements

- **FR-001**: Library MUST provide a `metric_exporter_adapter()` method on Python `OtlpLibrary` that returns a Python class implementing Python OpenTelemetry SDK's `MetricExporter` interface
- **FR-002**: The returned Python metric exporter adapter MUST implement all required methods of Python OpenTelemetry SDK's `MetricExporter` interface
- **FR-003**: The Python metric exporter adapter MUST delegate metric export operations to the underlying `OtlpLibrary` instance's Python API methods
- **FR-004**: The Python metric exporter adapter MUST convert Python OpenTelemetry SDK metric types to library-compatible formats before exporting
- **FR-005**: The Python metric exporter adapter MUST convert library errors (`OtlpError`) to appropriate Python OpenTelemetry SDK error types
- **FR-006**: Library MUST provide a `span_exporter_adapter()` method on Python `OtlpLibrary` that returns a Python class implementing Python OpenTelemetry SDK's `SpanExporter` interface
- **FR-007**: The returned Python span exporter adapter MUST implement all required methods of Python OpenTelemetry SDK's `SpanExporter` interface
- **FR-008**: The Python span exporter adapter MUST delegate span export operations to the underlying `OtlpLibrary` instance's Python API methods
- **FR-009**: The Python span exporter adapter MUST convert Python OpenTelemetry SDK span types to library-compatible formats before exporting
- **FR-010**: The Python span exporter adapter MUST convert library errors (`OtlpError`) to appropriate Python OpenTelemetry SDK error types
- **FR-011**: Adapter implementations MUST handle shutdown and flush methods from Python OpenTelemetry SDK gracefully, allowing Python OpenTelemetry SDK to manage lifecycle without requiring direct library shutdown
- **FR-012**: Adapter implementations MUST be usable directly with Python OpenTelemetry SDK's `PeriodicExportingMetricReader` (for metrics) and `BatchSpanProcessor` (for traces) without requiring wrapper code
- **FR-013**: Adapter implementations MUST support concurrent use from multiple Python OpenTelemetry SDK components
- **FR-014**: All adapter-related methods and types MUST function correctly on Windows, Linux, and macOS platforms
- **FR-015**: Python adapter implementations MUST support Python 3.11 or higher
- **FR-016**: Adapter implementations MUST handle errors gracefully, converting library errors to appropriate Python OpenTelemetry SDK error types without losing error context
- **FR-017**: Adapter implementations MUST not require `OtlpLibrary` shutdown to be called through the adapter - library shutdown remains a separate operation
- **FR-018**: Type conversion between Python OpenTelemetry SDK types and library-compatible formats MUST preserve all relevant metric and span data without loss or corruption
- **FR-019**: Adapter implementations MUST handle Python garbage collection appropriately, ensuring adapter objects remain valid while in use by Python OpenTelemetry SDK

### Key Entities *(include if feature involves data)*

- **Python Metric Exporter Adapter**: Represents a Python class that implements Python OpenTelemetry SDK's `MetricExporter` interface, wraps a Python `OtlpLibrary` instance, delegates export operations to the library's Python API methods, and provides seamless integration with Python OpenTelemetry SDK's metric collection system. Must handle type conversion between Python OpenTelemetry SDK metric types and library-compatible formats, error conversion, and lifecycle management independently of the library's shutdown process.

- **Python Span Exporter Adapter**: Represents a Python class that implements Python OpenTelemetry SDK's `SpanExporter` interface, wraps a Python `OtlpLibrary` instance, delegates export operations to the library's Python API methods, and provides seamless integration with Python OpenTelemetry SDK's trace collection system. Must handle type conversion between Python OpenTelemetry SDK span types and library-compatible formats, error conversion, and lifecycle management independently of the library's shutdown process.

- **Type Conversion Layer**: Represents the capability to convert between Python OpenTelemetry SDK types (metrics, spans) and library-compatible formats, enabling seamless data flow from Python OpenTelemetry SDK to the library's storage system. Must preserve all relevant data without loss or corruption while handling differences in data structures and representations between Python OpenTelemetry SDK and the library.

## Success Criteria *(mandatory)*

### Measurable Outcomes

- **SC-001**: Python developers can create and use metric exporter adapters with Python OpenTelemetry SDK's `PeriodicExportingMetricReader` in under 5 lines of code, eliminating the need for custom adapter implementations
- **SC-002**: Python developers can create and use span exporter adapters with Python OpenTelemetry SDK's `BatchSpanProcessor` and `TracerProvider` in under 5 lines of code, eliminating the need for custom adapter implementations
- **SC-003**: Adapter implementations successfully integrate with Python OpenTelemetry SDK in 95% of standard integration scenarios without requiring custom error handling or wrapper code
- **SC-004**: Adapter error conversion preserves error context in at least 90% of error scenarios, allowing developers to diagnose issues effectively
- **SC-005**: Type conversion between Python OpenTelemetry SDK types and library-compatible formats preserves 100% of metric and span data without loss or corruption
- **SC-006**: Adapter implementations function correctly on all three target platforms (Windows, Linux, macOS) with identical behavior and no platform-specific workarounds required
- **SC-007**: Adapter implementations handle concurrent use from multiple Python OpenTelemetry SDK components without data corruption or race conditions
- **SC-008**: Integration tests demonstrate that adapters work correctly with Python OpenTelemetry SDK's standard patterns (`PeriodicExportingMetricReader`, `BatchSpanProcessor`, `TracerProvider`) with 100% test pass rate
- **SC-009**: Python adapter implementations support Python 3.11+ and function correctly across different Python runtime environments
- **SC-010**: Adapter implementations handle Python garbage collection appropriately, with no memory leaks or invalid object references in 99% of usage scenarios
- **SC-011**: Python developers report that adapter implementations reduce integration time by at least 50% compared to creating custom adapter classes

## Assumptions

- Python OpenTelemetry SDK (`opentelemetry-python`) is a separate SDK from the Rust OpenTelemetry SDK and has its own exporter interfaces that differ from Rust SDK interfaces
- Python OpenTelemetry SDK's `MetricExporter` and `SpanExporter` interfaces are stable and will not change in ways that break compatibility during the supported Python OpenTelemetry SDK version lifecycle
- Python developers will call `OtlpLibrary::shutdown()` separately when their application shuts down, as adapter shutdown methods are independent of library lifecycle
- Type conversion between Python OpenTelemetry SDK types and library-compatible formats can be implemented efficiently without significant performance overhead
- Python OpenTelemetry SDK version compatibility: The adapters will target the current stable version of Python OpenTelemetry SDK, with testing on the most recent stable version available at implementation time
- Python developers have Python OpenTelemetry SDK installed in their environment when using the adapters
- The library's existing Python API methods (`export_metrics`, `export_traces`, `export_metrics_ref`) provide sufficient functionality for adapter implementations to delegate to

## Dependencies

- **Issue #5 Implementation (003-opentelemetry-exporters)**: The Rust-side exporter implementations (Issue #5) provide the pattern and foundation for adapter implementations, though Python adapters implement different interfaces (Python OpenTelemetry SDK vs Rust OpenTelemetry SDK)
- **Python Bindings Infrastructure**: Requires existing Python bindings infrastructure (PyO3) to be in place for Python API exposure
- **Python OpenTelemetry SDK Research**: Requires understanding of Python OpenTelemetry SDK exporter interfaces (`opentelemetry.sdk.metrics.export.MetricExporter`, `opentelemetry.sdk.trace.export.SpanExporter`) and patterns
- **Cross-platform Compatibility**: Requires that the underlying `OtlpLibrary` functionality works correctly on all target platforms, as adapters depend on library functionality
- **Type Conversion Implementation**: Requires implementation of type conversion logic between Python OpenTelemetry SDK types and library-compatible formats

## Out of Scope

- Supporting Python OpenTelemetry SDK versions other than current stable (version compatibility can be addressed in future enhancements)
- Custom adapter configurations beyond what `OtlpLibrary` provides
- Adapters for other Python OpenTelemetry SDK exporter types (only `MetricExporter` and `SpanExporter` initially)
- Performance optimizations beyond efficient type conversion (additional optimizations can be added in future)
- Adapter implementations that work with Python versions below 3.11 (Python 3.11+ is the minimum supported version)
- Automatic Python OpenTelemetry SDK version detection or compatibility checking (developers are responsible for using compatible versions)
