# Feature Specification: Built-in OpenTelemetry Exporter Implementations

**Feature Branch**: `003-opentelemetry-exporters`  
**Created**: 2025-01-27  
**Status**: Draft  
**Input**: User description: "Review the following Github issues: [Issue #5](https://github.com/pixie79/otlp-rust-service/issues/5) and [Issue #4](https://github.com/pixie79/otlp-rust-service/issues/4). Remember cross compatibility requirements for osx, windows, linux along with our requirement for exposing methods to be callable via rust and python"

## User Scenarios & Testing *(mandatory)*

### User Story 1 - Efficient Metrics Export with Reference Semantics (Priority: P1)

A developer integrating the library with OpenTelemetry SDK needs to export metrics without unnecessary data copying. The system must provide a method that accepts metrics by reference, allowing efficient integration with OpenTelemetry SDK's periodic readers that pass metrics as references rather than owned values.

**Why this priority**: This is a foundational requirement that enables efficient integration with OpenTelemetry SDK. Without this, developers must work around the limitation by cloning data, which is inefficient and error-prone. This story provides the core capability needed for the exporter implementations in Story 2.

**Independent Test**: Can be fully tested by calling the reference-based export method with metrics data and verifying that metrics are correctly processed and stored without requiring data ownership transfer. The test delivers an efficient export mechanism that eliminates unnecessary copying.

**Acceptance Scenarios**:

1. **Given** an `OtlpLibrary` instance is created, **When** a developer calls `export_metrics_ref` with a reference to `ResourceMetrics`, **Then** the metrics are processed and buffered without requiring ownership of the data
2. **Given** metrics are exported via the reference method, **When** the library processes them, **Then** the metrics are written to disk in the same format and location as metrics exported via the owned method
3. **Given** metrics are exported via the reference method, **When** the library processes them, **Then** no data is lost or corrupted compared to the owned export method
4. **Given** the reference export method is called from Rust code, **When** metrics are passed by reference, **Then** the method accepts the reference and processes metrics correctly
5. **Given** the reference export method is exposed via Python bindings, **When** Python code calls the method, **Then** the method accepts Python-provided metrics data and processes them correctly

---

### User Story 2 - Direct Integration with OpenTelemetry SDK (Priority: P2)

A developer wants to integrate the library directly with OpenTelemetry SDK without writing custom wrapper code. The system must provide ready-to-use exporter implementations (`PushMetricExporter` and `SpanExporter`) that can be used directly with OpenTelemetry SDK's readers and providers, eliminating the need for boilerplate wrapper implementations.

**Why this priority**: This delivers the primary value proposition - seamless integration with OpenTelemetry SDK. While Story 1 provides the foundation, this story enables the complete integration experience that eliminates boilerplate and ensures consistent behavior across all consumers. This is the feature that makes the library production-ready for OpenTelemetry SDK integration.

**Independent Test**: Can be fully tested by creating exporter instances from an `OtlpLibrary` and using them directly with OpenTelemetry SDK's `PeriodicReader` and `TracerProvider` without any custom wrapper code. The test delivers a complete, working integration that demonstrates the library's ease of use.

**Acceptance Scenarios**:

1. **Given** an `OtlpLibrary` instance is created, **When** a developer calls `metric_exporter()` method, **Then** the method returns a `PushMetricExporter` implementation that can be used directly with OpenTelemetry SDK's `PeriodicReader`
2. **Given** an `OtlpLibrary` instance is created, **When** a developer calls `span_exporter()` method, **Then** the method returns a `SpanExporter` implementation that can be used directly with OpenTelemetry SDK's `TracerProvider`
3. **Given** a metric exporter is created from the library, **When** it is used with OpenTelemetry SDK's `PeriodicReader`, **Then** metrics are automatically exported to the library's storage system at the configured intervals
4. **Given** a span exporter is created from the library, **When** it is used with OpenTelemetry SDK's `TracerProvider`, **Then** spans are automatically exported to the library's storage system when batches are ready
5. **Given** exporters are created from the library, **When** they are used with OpenTelemetry SDK, **Then** errors from the library are properly converted to OpenTelemetry SDK error types
6. **Given** exporters are created from the library, **When** OpenTelemetry SDK calls shutdown methods, **Then** the exporters handle shutdown gracefully without affecting the library's lifecycle
7. **Given** a metric exporter is created, **When** OpenTelemetry SDK queries temporality, **Then** the exporter returns the appropriate temporality value (defaulting to Cumulative)

---

### User Story 3 - Python API Access to Exporters (Priority: P3)

A Python developer wants to use the built-in exporter implementations from Python code. The system must expose the exporter creation methods and exporter types through Python bindings, allowing Python applications to integrate with OpenTelemetry SDK using the same convenience methods available in Rust.

**Why this priority**: This extends the convenience of built-in exporters to Python users, ensuring feature parity between Rust and Python APIs. While the core functionality works in Rust, Python users should have the same seamless integration experience. This story completes the cross-language support requirement.

**Independent Test**: Can be fully tested by creating exporter instances from Python code and verifying that they can be used with Python OpenTelemetry SDK bindings. The test delivers Python developers the same integration convenience as Rust developers.

**Acceptance Scenarios**:

1. **Given** a Python `OtlpLibrary` instance is created, **When** a Python developer calls `metric_exporter()` method, **Then** the method returns a Python-compatible metric exporter object
2. **Given** a Python `OtlpLibrary` instance is created, **When** a Python developer calls `span_exporter()` method, **Then** the method returns a Python-compatible span exporter object
3. **Given** Python exporter objects are created, **When** they are used with Python OpenTelemetry SDK, **Then** metrics and spans are exported correctly to the library's storage system
4. **Given** Python exporter objects are created, **When** they are used across different Python versions (3.11+), **Then** they function correctly on all supported Python versions
5. **Given** Python exporter objects are created, **When** they are used on different operating systems (Windows, Linux, macOS), **Then** they function correctly on all supported platforms

---

### Edge Cases

- What happens when `export_metrics_ref` is called with an empty or invalid `ResourceMetrics` reference?
- How does the system handle concurrent calls to `export_metrics_ref` from multiple threads?
- What happens when an exporter is created from a library instance that has been shut down?
- How does the system handle errors when exporters attempt to export data after the library is shut down?
- What happens when OpenTelemetry SDK calls exporter methods with invalid or malformed data?
- How does the system handle memory pressure when exporting large batches of metrics or spans via reference?
- What happens when `export_metrics_ref` is called but the internal buffer is full?
- How does the system handle shutdown timeouts when OpenTelemetry SDK requests graceful shutdown?
- What happens when temporality is queried from a metric exporter that hasn't been configured?
- How does the system handle Python garbage collection of exporter objects while they're still in use by OpenTelemetry SDK?
- What happens when Python bindings are called from different Python threads or async contexts?
- How does the system handle platform-specific differences in reference handling between Windows, Linux, and macOS?

## Requirements *(mandatory)*

### Functional Requirements

- **FR-001**: Library MUST provide an `export_metrics_ref` method on `OtlpLibrary` that accepts a reference to `ResourceMetrics` (`&ResourceMetrics`) instead of requiring ownership
- **FR-002**: The `export_metrics_ref` method MUST process metrics identically to the existing `export_metrics` method, ensuring no functional differences between reference and owned variants
- **FR-003**: The `export_metrics_ref` method MUST be callable from Rust code using standard Rust reference semantics
- **FR-004**: The `export_metrics_ref` method MUST be exposed via Python bindings and callable from Python code
- **FR-005**: Library MUST provide a `metric_exporter()` method on `OtlpLibrary` that returns a `PushMetricExporter` implementation
- **FR-006**: The returned `PushMetricExporter` MUST implement all required methods of the `PushMetricExporter` trait from OpenTelemetry SDK
- **FR-007**: The `PushMetricExporter` implementation MUST delegate metric export operations to the underlying `OtlpLibrary` instance's `export_metrics_ref` method
- **FR-008**: The `PushMetricExporter` implementation MUST convert `OtlpError` to `OTelSdkError` for compatibility with OpenTelemetry SDK error handling
- **FR-009**: The `PushMetricExporter` implementation MUST return `Cumulative` temporality by default when queried
- **FR-010**: Library MUST provide a `span_exporter()` method on `OtlpLibrary` that returns a `SpanExporter` implementation
- **FR-011**: The returned `SpanExporter` MUST implement all required methods of the `SpanExporter` trait from OpenTelemetry SDK
- **FR-012**: The `SpanExporter` implementation MUST delegate span export operations to the underlying `OtlpLibrary` instance's `export_traces` method
- **FR-013**: The `SpanExporter` implementation MUST convert `OtlpError` to `OTelSdkError` for compatibility with OpenTelemetry SDK error handling
- **FR-014**: Exporter implementations MUST handle shutdown methods gracefully, allowing OpenTelemetry SDK to manage lifecycle without requiring direct library shutdown
- **FR-015**: Exporter implementations MUST be usable directly with OpenTelemetry SDK's `PeriodicReader` (for metrics) and `TracerProvider` (for traces) without requiring wrapper code
- **FR-016**: Exporter implementations MUST support concurrent use from multiple OpenTelemetry SDK components
- **FR-017**: The `metric_exporter()` method MUST be exposed via Python bindings, returning a Python-compatible metric exporter object
- **FR-018**: The `span_exporter()` method MUST be exposed via Python bindings, returning a Python-compatible span exporter object
- **FR-019**: Python exporter objects MUST be usable with Python OpenTelemetry SDK bindings
- **FR-020**: All exporter-related methods and types MUST function correctly on Windows, Linux, and macOS platforms
- **FR-021**: Python bindings for exporters MUST support Python 3.11 or higher
- **FR-022**: Exporter implementations MUST handle errors gracefully, converting library errors to appropriate OpenTelemetry SDK error types without losing error context
- **FR-023**: Exporter implementations MUST not require `OtlpLibrary` shutdown to be called through the exporter - library shutdown remains a separate operation

### Key Entities *(include if feature involves data)*

- **Metric Exporter**: Represents a `PushMetricExporter` implementation that wraps an `OtlpLibrary` instance, delegates export operations to the library's `export_metrics_ref` method, and provides seamless integration with OpenTelemetry SDK's metric collection system. Must handle error conversion and lifecycle management independently of the library's shutdown process.

- **Span Exporter**: Represents a `SpanExporter` implementation that wraps an `OtlpLibrary` instance, delegates export operations to the library's `export_traces` method, and provides seamless integration with OpenTelemetry SDK's trace collection system. Must handle error conversion and lifecycle management independently of the library's shutdown process.

- **Reference-based Export**: Represents the capability to export metrics by reference rather than by ownership, enabling efficient integration with OpenTelemetry SDK components that pass data by reference. Must maintain functional equivalence with owned export methods while avoiding unnecessary data copying.

## Success Criteria *(mandatory)*

### Measurable Outcomes

- **SC-001**: Developers can create and use metric exporters with OpenTelemetry SDK's `PeriodicReader` in under 5 lines of code, eliminating the need for custom wrapper implementations
- **SC-002**: Developers can create and use span exporters with OpenTelemetry SDK's `TracerProvider` in under 5 lines of code, eliminating the need for custom wrapper implementations
- **SC-003**: Metrics exported via `export_metrics_ref` are processed and stored with 100% functional equivalence to metrics exported via `export_metrics` (owned method)
- **SC-004**: Exporter implementations successfully integrate with OpenTelemetry SDK in 95% of standard integration scenarios without requiring custom error handling or wrapper code
- **SC-005**: Exporter error conversion preserves error context in at least 90% of error scenarios, allowing developers to diagnose issues effectively
- **SC-006**: Python developers can create and use exporters from Python code with the same ease as Rust developers, with feature parity between Rust and Python APIs
- **SC-007**: Exporter implementations function correctly on all three target platforms (Windows, Linux, macOS) with identical behavior and no platform-specific workarounds required
- **SC-008**: Exporter implementations handle concurrent use from multiple OpenTelemetry SDK components without data corruption or race conditions
- **SC-009**: Reference-based export method reduces memory allocations by at least 50% compared to owned export method when used with OpenTelemetry SDK's periodic readers
- **SC-010**: Integration tests demonstrate that exporters work correctly with OpenTelemetry SDK's standard patterns (PeriodicReader, TracerProvider) with 100% test pass rate
- **SC-011**: Python bindings for exporters support Python 3.11+ and function correctly across different Python runtime environments

## Assumptions

- OpenTelemetry SDK version 0.31 is the target version for compatibility (matching current library dependencies)
- The `ResourceMetrics` type from OpenTelemetry SDK does not implement `Clone`, necessitating the reference-based export method
- OpenTelemetry SDK's `PushMetricExporter` and `SpanExporter` traits are stable and will not change in ways that break compatibility
- Python has a separate OpenTelemetry SDK (opentelemetry-python) that is independent from the Rust SDK. Python bindings for our library expose `OtlpLibrary` to Python, but Python developers would need to bridge between Python OpenTelemetry SDK types and our library's Python API. This may require additional Python wrapper classes that implement Python OpenTelemetry SDK exporter interfaces, or documentation on how to create such bridges.
- Developers using exporters will call `OtlpLibrary::shutdown()` separately when their application shuts down, as exporter shutdown methods are independent of library lifecycle
- The default `Cumulative` temporality for metric exporters is acceptable for most use cases and can be made configurable in future enhancements if needed

## Dependencies

- **Issue #4 Implementation**: The `export_metrics_ref` method (Issue #4) must be implemented before the `PushMetricExporter` implementation (Issue #5) can be completed, as the exporter depends on this method
- **OpenTelemetry SDK**: Requires OpenTelemetry SDK 0.31 or compatible version for trait implementations
- **Python Bindings Infrastructure**: Requires existing Python bindings infrastructure (PyO3) to be in place for Python API exposure
- **Cross-platform Compatibility**: Requires that the underlying `OtlpLibrary` functionality works correctly on all target platforms, as exporters depend on library functionality

## Python OpenTelemetry SDK Integration Clarification

**Important Note**: Python has its own separate OpenTelemetry SDK (`opentelemetry-python`) that is independent from the Rust OpenTelemetry SDK. The Python bindings we provide expose our Rust `OtlpLibrary` to Python, but do not directly implement Python OpenTelemetry SDK exporter interfaces.

**For Python Users**:
- Python developers can use our library's Python API methods (`export_metrics`, `export_traces`) directly
- To integrate with Python OpenTelemetry SDK, Python developers would need to create adapter classes that:
  - Implement Python OpenTelemetry SDK exporter interfaces (`MetricExporter`, `SpanExporter`)
  - Call our library's Python API methods internally
- This is similar to the situation Rust developers face today (creating wrapper types), but in Python

**Future Consideration**:
- We may want to provide Python adapter classes that implement Python OpenTelemetry SDK interfaces
- This would require understanding Python OpenTelemetry SDK patterns and may be a separate enhancement
- **Tracked in**: [GitHub Issue #6](https://github.com/pixie79/otlp-rust-service/issues/6) - Python OpenTelemetry SDK Adapter Classes
- For now, Python users have access to the core library functionality via Python bindings

## Out of Scope

- Configurable temporality for metric exporters (defaults to Cumulative, see analysis below)
- Custom exporter configuration beyond what `OtlpLibrary` provides
- Exporter implementations for other OpenTelemetry SDK exporter types (only `PushMetricExporter` and `SpanExporter` are in scope)
- Performance optimizations beyond reference-based export (additional optimizations can be added in future)
- Exporter implementations that work with OpenTelemetry SDK versions other than 0.31 (version compatibility can be addressed in future)

## Temporality Configurability Analysis

**Question**: Should configurable temporality be included in the initial implementation?

**Current Decision**: Defer to future enhancement (defaults to Cumulative)

**Analysis**:

**Arguments for including now:**
- Simple addition: Temporality is a single method return value (`temporality() -> Temporality`)
- Low implementation cost: Just add a configuration option and use it in the exporter
- Better API completeness: Makes the exporter more flexible from the start
- Avoids breaking changes: Adding it later might require API changes

**Arguments for deferring:**
- Cumulative is the default and most common use case (covers 90%+ of scenarios)
- Keeps initial implementation focused and simpler
- Can be added as a non-breaking enhancement later (just add a builder method)
- Reduces initial complexity and testing surface area

**Recommendation**: **Defer to future enhancement** - The implementation is straightforward enough that it can be added later without breaking changes. Cumulative temporality covers the vast majority of use cases, and keeping the initial implementation focused allows for faster delivery of core value. However, if there's a specific use case requiring Delta temporality, it can be prioritized.

**Future Enhancement Path**: Add a builder method like `OtlpMetricExporter::with_temporality(Temporality)` or a configuration option on `OtlpLibrary` that gets passed to the exporter.

**Tracked in**: [GitHub Issue #9](https://github.com/pixie79/otlp-rust-service/issues/9) - Configurable Temporality for Metric Exporters

## Future Enhancement Opportunities

The following GitHub issues have been created to track potential future enhancements:

### GitHub Issue #7: Support Additional OpenTelemetry SDK Exporter Types

**Status**: [Created](https://github.com/pixie79/otlp-rust-service/issues/7)

Currently, the library provides built-in implementations for `PushMetricExporter` and `SpanExporter`. This issue tracks potential support for other OpenTelemetry SDK exporter types that may be useful for different integration scenarios.

**Potential Exporter Types to Consider**:
- `PullMetricExporter` - For pull-based metric collection scenarios
- `LogRecordExporter` - For log data export (when OpenTelemetry SDK log support is stable)
- Custom exporter interfaces that may emerge in future SDK versions

**Out of Scope for Current Feature**:
- Only `PushMetricExporter` and `SpanExporter` are in scope for the initial implementation
- This issue serves as a placeholder for future expansion based on user needs and SDK evolution

---

### GitHub Issue #8: Performance Optimizations for Exporter Implementations

**Status**: [Created](https://github.com/pixie79/otlp-rust-service/issues/8)

This issue tracks potential performance optimizations for the built-in exporter implementations beyond the reference-based export method (`export_metrics_ref`).

**Potential Optimizations to Consider**:
- Batch size optimization for exporter operations
- Async operation batching and coalescing
- Memory pool/reuse strategies for frequently allocated types
- Zero-copy data transfer optimizations where possible
- Connection pooling for remote forwarding scenarios (if applicable)
- Lock-free data structures for high-concurrency scenarios
- SIMD optimizations for metric aggregation operations

**Out of Scope for Current Feature**:
- Initial implementation focuses on correctness and API completeness
- Performance optimizations should be data-driven based on actual usage patterns and profiling
- This issue serves as a placeholder for future optimization work
