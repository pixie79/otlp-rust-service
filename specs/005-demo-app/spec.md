# Feature Specification: Demo Rust Application

**Feature Branch**: `005-demo-app`  
**Created**: 2025-11-26  
**Status**: Draft  
**Input**: User description: "Create a demo rust app, this should enable the dashboard then create mock metrics and log span messages. This will allow developers to a) prove the service is working and b) be a reference implementation for developers to follow when using the SDK"

## User Scenarios & Testing *(mandatory)*

### User Story 1 - Verify Service Functionality (Priority: P1)

A developer needs to quickly verify that the OTLP service is working correctly after installation or configuration changes. The demo application must demonstrate that the service can receive telemetry data, process it, and make it visible through the dashboard.

**Why this priority**: This is the primary purpose of the demo - proving the service works. Without this capability, developers cannot confidently verify their setup is correct. This story provides immediate value by allowing developers to validate the entire system end-to-end.

**Independent Test**: Can be fully tested by running the demo application and verifying that metrics and spans appear in the dashboard within a reasonable time frame. The test delivers confidence that the service is operational and correctly configured.

**Acceptance Scenarios**:

1. **Given** the demo application is started with dashboard enabled, **When** the application generates mock metrics and spans, **Then** the data appears in the dashboard within the configured write interval
2. **Given** the demo application is running, **When** a developer opens the dashboard in a web browser, **Then** they can see real-time metrics and trace data being displayed
3. **Given** the demo application generates telemetry data, **When** the data is exported, **Then** it is stored in Arrow IPC format files in the configured output directory
4. **Given** the demo application is running, **When** it generates continuous mock data, **Then** the dashboard updates automatically showing new data as it arrives
5. **Given** the demo application completes execution, **When** a developer checks the output directory, **Then** they find Arrow IPC files containing the generated metrics and traces

---

### User Story 2 - Reference Implementation for SDK Usage (Priority: P2)

A developer needs a clear, working example of how to use the OTLP SDK to create and export metrics and spans in their own application. The demo application must demonstrate best practices and common patterns for SDK integration.

**Why this priority**: While proving the service works is critical, providing a reference implementation is equally important for adoption. Developers need concrete examples they can follow and adapt. This story enables developers to quickly integrate the SDK into their own applications.

**Independent Test**: Can be fully tested by examining the demo application code and verifying it demonstrates all key SDK usage patterns (initialization, metric creation, span creation, export, shutdown). The test delivers a complete reference that developers can use as a starting point.

**Acceptance Scenarios**:

1. **Given** a developer examines the demo application code, **When** they review the implementation, **Then** they can see clear examples of how to initialize the OTLP library with dashboard enabled
2. **Given** a developer examines the demo application code, **When** they review metric creation, **Then** they can see examples of creating different types of metrics with appropriate attributes
3. **Given** a developer examines the demo application code, **When** they review span creation, **Then** they can see examples of creating spans with different kinds, attributes, and relationships
4. **Given** a developer examines the demo application code, **When** they review the export process, **Then** they can see examples of exporting both individual and batch telemetry data
5. **Given** a developer uses the demo application as a template, **When** they adapt it for their own use case, **Then** they can successfully integrate the SDK following the demonstrated patterns
6. **Given** the demo application code is well-documented, **When** a developer reads the code comments, **Then** they understand the purpose and usage of each SDK method call

---

### User Story 3 - Continuous Data Generation for Testing (Priority: P3)

A developer needs to test dashboard functionality and data visualization with a continuous stream of realistic telemetry data. The demo application must generate mock data over time to simulate real-world usage patterns.

**Why this priority**: While generating a single batch of data proves functionality, continuous generation enables testing of dashboard real-time features, data aggregation, and visualization updates. This story enhances the demo's utility for testing and demonstration purposes.

**Independent Test**: Can be fully tested by running the demo application and verifying that it generates data continuously over a specified duration, with metrics and spans appearing at regular intervals. The test delivers a realistic data stream for dashboard testing.

**Acceptance Scenarios**:

1. **Given** the demo application is configured to run continuously, **When** it executes, **Then** it generates metrics and spans at regular intervals (e.g., every few seconds)
2. **Given** the demo application generates continuous data, **When** it runs for an extended period, **Then** the dashboard displays time-series data showing trends and patterns
3. **Given** the demo application generates mock data, **When** it creates metrics, **Then** the metrics include realistic values that change over time to demonstrate visualization capabilities
4. **Given** the demo application generates mock data, **When** it creates spans, **Then** the spans include realistic attributes and relationships that demonstrate trace visualization
5. **Given** the demo application is running continuously, **When** a developer stops it, **Then** it shuts down gracefully, flushing any pending data

---

### Edge Cases

- What happens when the dashboard port is already in use?
- How does the demo handle errors when creating or exporting metrics?
- How does the demo handle errors when creating or exporting spans?
- What happens when the output directory is not writable?
- How does the demo behave if the dashboard fails to start?
- What happens when the demo is interrupted (Ctrl+C) while generating data?
- How does the demo handle configuration errors (e.g., invalid dashboard port)?
- What happens if the demo generates data faster than the write interval?
- How does the demo handle resource exhaustion (e.g., too many open files)?
- What happens when the demo runs on a system with limited disk space?

## Requirements *(mandatory)*

### Functional Requirements

- **FR-001**: Demo application MUST enable the dashboard by default when started
- **FR-002**: Demo application MUST create and export mock metrics using the OTLP SDK
- **FR-003**: Demo application MUST create and export mock span messages using the OTLP SDK
- **FR-004**: Demo application MUST demonstrate dashboard configuration (enabling dashboard, setting port, output directory)
- **FR-005**: Demo application MUST demonstrate library initialization with dashboard enabled
- **FR-006**: Demo application MUST demonstrate creating metrics with realistic attributes and values
- **FR-007**: Demo application MUST demonstrate creating spans with different span kinds (server, client, internal)
- **FR-008**: Demo application MUST demonstrate creating spans with realistic attributes (service name, HTTP method, status codes, etc.)
- **FR-009**: Demo application MUST demonstrate exporting individual metrics
- **FR-010**: Demo application MUST demonstrate exporting individual spans
- **FR-011**: Demo application MUST demonstrate batch exporting of multiple metrics
- **FR-012**: Demo application MUST demonstrate batch exporting of multiple spans
- **FR-013**: Demo application MUST include clear code comments explaining each SDK usage pattern
- **FR-014**: Demo application MUST be runnable as a standalone executable (e.g., `cargo run --example demo-app`)
- **FR-015**: Demo application MUST generate data that is visible in the dashboard within the configured write interval
- **FR-016**: Demo application MUST demonstrate graceful shutdown (flushing pending data before exit)
- **FR-017**: Demo application MUST use realistic mock data that demonstrates dashboard visualization capabilities
- **FR-018**: Demo application MUST generate metrics that show time-series patterns (e.g., counter increments, gauge fluctuations)
- **FR-019**: Demo application MUST generate spans that demonstrate trace relationships (parent-child spans, trace context propagation)
- **FR-020**: Demo application MUST be well-documented with inline comments explaining the purpose of each code section
- **FR-021**: Demo application MUST demonstrate error handling for common failure scenarios (optional, can be basic)
- **FR-022**: Demo application MUST be located in the examples directory following project conventions
- **FR-023**: Demo application MUST be compilable and runnable without additional dependencies beyond the OTLP library

### Key Entities *(include if feature involves data)*

- **Demo Application**: Represents a standalone Rust application that demonstrates OTLP SDK usage, includes dashboard configuration, metric generation, span generation, and data export patterns, serves as both a verification tool and reference implementation
- **Mock Metrics**: Represents synthetic metric data created for demonstration purposes, includes realistic values and attributes that showcase dashboard visualization capabilities, demonstrates different metric types (counters, gauges, histograms if supported)
- **Mock Spans**: Represents synthetic trace span data created for demonstration purposes, includes realistic attributes (service names, HTTP methods, status codes), demonstrates different span kinds and relationships, showcases trace visualization in the dashboard
- **Dashboard Configuration**: Represents the settings needed to enable and configure the dashboard, includes port number, static file directory, and bind address, must be demonstrated in the demo application code

## Success Criteria *(mandatory)*

### Measurable Outcomes

- **SC-001**: Developers can run the demo application and see metrics and spans appear in the dashboard within 10 seconds of starting the application
- **SC-002**: The demo application successfully generates and exports at least 10 distinct metrics and 10 distinct spans during a single execution
- **SC-003**: Developers can use the demo application code as a reference to integrate the SDK into their own application within 30 minutes
- **SC-004**: The demo application code contains clear comments explaining at least 80% of SDK method calls and configuration options
- **SC-005**: The demo application compiles and runs successfully on all supported platforms (Windows, Linux, macOS) without modification
- **SC-006**: The demo application demonstrates all primary SDK usage patterns (initialization, metric creation, span creation, export, shutdown) in a single, cohesive example
- **SC-007**: Developers can verify service functionality by running the demo and confirming data appears in the dashboard with 100% success rate under normal conditions
- **SC-008**: The demo application generates data that creates visible, meaningful visualizations in the dashboard (not just empty or minimal data)
- **SC-009**: The demo application code follows Rust best practices and is readable by developers with intermediate Rust knowledge
- **SC-010**: The demo application demonstrates realistic telemetry data patterns that help developers understand how to structure their own metrics and spans

## Assumptions

- The dashboard static files are available in the default location (`./dashboard/dist`) or can be configured
- Developers have basic familiarity with Rust and can read and understand example code
- The demo application will be run in a development environment where the dashboard port (default 8080) is available
- The demo application does not need to demonstrate advanced features like forwarding or custom authentication
- The demo application focuses on the most common SDK usage patterns rather than exhaustive coverage of all features
- Mock data generation can be simple and periodic rather than complex event-driven patterns
- The demo application can use default configuration values for most settings, only demonstrating dashboard enablement explicitly
