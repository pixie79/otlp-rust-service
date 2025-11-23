# Feature Specification: OTLP Arrow Flight Library

**Feature Branch**: `001-otlp-arrow-library`  
**Created**: 2024-11-23  
**Status**: Draft  
**Input**: User description: "Build a cross platform rust libary that can be used either standalone and by exposing public method that could be included into another application. It should be capable of recieving OTLP messages using gRPC with ARROW Flight IPC in the first instance (we will enhance later to also allow gRPC with Protobuf. It should be capable of writing the OTLP messages into a category specified local output dir, i,e {OUTPUT_DIR}/otlp/metrics, {OUTPUT_DIR}/otlp/traces. These output files should be in Arrow IPC Streaming format. - We should offer a config option for setting the {OUTPUT_DIR} (default ./output_dir, that should also include options to set how frequently we write trace/metric Arrow batches to disk the default should be every 5 seconds. We should also include a config option on how frequently we should clean out both the metrics files and the traces. default for traces is 600s for metrics is 1h. We should be able to forward OTLP metrics / traces onto a remote OTLP_Endpoint service using gRPC with Arrow Flight IPC messages. (In future we will extend this to also support gRPC with Proto). This should be able to be configured with authentication as needed by the standard. The forwarding should be optional and disabled by default, configuration should be via the same config process above"

## User Scenarios & Testing *(mandatory)*

### User Story 1 - Core OTLP Ingestion and Local Storage (Priority: P1)

A developer or application needs to receive OpenTelemetry Protocol (OTLP) messages and persist them locally for later analysis or processing. The system must accept incoming OTLP messages via the standard protocol, organize them by type (metrics and traces), and write them to local storage in a format that preserves the data structure and can be efficiently processed later.

**Why this priority**: This is the core functionality that delivers the primary value - capturing and storing telemetry data. Without this, the library has no purpose. This story provides a complete, usable MVP that can be tested and demonstrated independently.

**Independent Test**: Can be fully tested by sending OTLP messages to the library and verifying that files are created in the expected directory structure with valid data. The test delivers a working system that can receive and store telemetry data, which is the minimum viable product.

**Acceptance Scenarios**:

1. **Given** the library is running with default configuration, **When** OTLP metrics messages are received via the standard protocol, **Then** the messages are written to files in the default output directory under the metrics subdirectory in Arrow IPC Streaming format
2. **Given** the library is running with default configuration, **When** OTLP trace messages are received via the standard protocol, **Then** the messages are written to files in the default output directory under the traces subdirectory in Arrow IPC Streaming format
3. **Given** the library is running, **When** messages are received continuously, **Then** messages are batched and written to disk at regular intervals (default 5 seconds) without data loss
4. **Given** the library is embedded in another application (Rust or Python), **When** the application calls the public API methods, **Then** the library processes OTLP messages and writes them to the configured output location
5. **Given** the library is running as a standalone service, **When** it receives OTLP messages, **Then** it processes and stores them without requiring external dependencies

---

### User Story 2 - Configuration System (Priority: P2)

A developer or operator needs to customize the library's behavior to match their environment and requirements. The system must provide a configuration mechanism that allows setting output directory, write frequency, and cleanup schedules without code changes.

**Why this priority**: While the MVP works with defaults, real-world usage requires customization. Different environments need different output locations, and operational requirements vary for write frequency and data retention. This story enhances the core functionality to be production-ready.

**Independent Test**: Can be fully tested by configuring the library with different settings and verifying that all configuration options are respected. The test delivers a configurable system that adapts to different deployment scenarios.

**Acceptance Scenarios**:

1. **Given** a configuration specifies a custom output directory, **When** the library processes messages, **Then** files are written to the specified directory instead of the default
2. **Given** a configuration specifies a custom write interval (e.g., 10 seconds), **When** messages are received, **Then** batches are written to disk at the specified interval
3. **Given** a configuration specifies cleanup intervals for traces (e.g., 300 seconds), **When** the library runs, **Then** trace files older than the specified interval are removed
4. **Given** a configuration specifies cleanup intervals for metrics (e.g., 30 minutes), **When** the library runs, **Then** metric files older than the specified interval are removed
5. **Given** configuration is provided via the standard configuration mechanism, **When** the library starts, **Then** all configuration values are validated and applied, with defaults used for unspecified values

---

### User Story 3 - Testing and Development Support (Priority: P3)

A developer needs to test the library's functionality end-to-end to ensure it works correctly in both usage modes (gRPC interface and public API methods). The system must provide a mock service that simulates OTLP message generation and reception, allowing comprehensive testing without external dependencies.

**Why this priority**: While the library can be tested with external tools, having a built-in mock service significantly improves developer experience and enables reliable, repeatable end-to-end testing. This ensures both integration paths (gRPC and public API) are validated together, catching integration issues early.

**Independent Test**: Can be fully tested by using the mock service to send messages via both gRPC interface and public API methods, then verifying that messages are correctly processed and stored. The test delivers confidence that the library works correctly in all usage scenarios.

**Acceptance Scenarios**:

1. **Given** the mock service is running, **When** messages are sent via the gRPC interface (Protobuf or Arrow Flight), **Then** the mock service receives and processes them, allowing validation of the complete gRPC message flow
2. **Given** the mock service is running, **When** messages are sent via the public API methods, **Then** the mock service receives and processes them, allowing validation of the complete programmatic usage flow
3. **Given** the mock service is used for testing, **When** end-to-end tests are executed, **Then** both gRPC protocols (Protobuf and Arrow Flight) and public API integration paths are validated without requiring external OTLP service dependencies
4. **Given** the mock service is configured, **When** it generates test messages, **Then** the messages follow OTLP protocol standards and can be used to validate library behavior
5. **Given** the mock service is running, **When** messages are sent via both gRPC protocols (Protobuf and Arrow Flight), **Then** the mock service successfully receives and processes messages from both protocols

---

### User Story 4 - Optional Remote Forwarding (Priority: P4)

An operator needs to forward collected telemetry data to a remote OTLP endpoint for centralized processing or analysis. The system must support optional forwarding to remote services with configurable authentication, while maintaining local storage as the primary function.

**Why this priority**: This adds valuable functionality for distributed systems and centralized observability, but the library is fully functional without it. This story extends the core capabilities for users who need remote forwarding, but it's not required for the MVP to deliver value.

**Independent Test**: Can be fully tested by enabling forwarding configuration and verifying that messages are both stored locally and forwarded to the remote endpoint. The test delivers a system that can operate in both local-only and hybrid (local + remote) modes.

**Acceptance Scenarios**:

1. **Given** forwarding is disabled (default), **When** messages are received, **Then** they are only stored locally and not sent to any remote endpoint
2. **Given** forwarding is enabled with a remote endpoint URL, **When** messages are received, **Then** they are stored locally and also forwarded to the specified remote endpoint
3. **Given** forwarding is enabled with authentication credentials, **When** messages are forwarded, **Then** the authentication is applied according to the OTLP standard
4. **Given** forwarding is enabled but the remote endpoint is unavailable, **When** messages are received, **Then** local storage continues to work, and forwarding failures are handled gracefully without blocking message processing
5. **Given** forwarding is configured via the same configuration mechanism as other settings, **When** the library starts, **Then** forwarding settings are validated and applied
6. **Given** forwarding is configured with a specific output format (Protobuf or Arrow Flight), **When** messages are received in a different format, **Then** messages are converted to the configured forwarding format before being sent to the remote endpoint

---

### Edge Cases

- What happens when the output directory does not exist or cannot be created?
- What happens when disk space is exhausted while writing files?
- How does the system handle malformed or invalid OTLP messages?
- What happens when write operations fail due to file system errors?
- How does the system handle concurrent writes to the same output files?
- What happens when cleanup operations fail (e.g., file locked or permission denied)?
- How does the system handle network failures when forwarding to remote endpoints?
- What happens when authentication fails for remote forwarding?
- How does format conversion work when forwarding messages in a different format than received?
- What happens if format conversion fails during forwarding?
- How does the system handle partial message batches during write intervals?
- What happens when the library is shut down while messages are being processed?
- How does the system handle very high message rates that exceed write frequency?
- What happens when configuration values are invalid (e.g., negative intervals, invalid paths)?
- How does the mock service handle invalid or malformed test messages?
- What happens when the mock service is used concurrently with real OTLP clients?

## Clarifications

### Session 2024-11-23

- Q: What languages must be able to call the public API methods? → A: Public methods must be callable from both Rust projects and Python projects
- Q: How should the library handle both gRPC with Protobuf and gRPC with Arrow Flight protocols? → A: Support both protocols simultaneously on different ports/endpoints, with configuration to enable/disable each independently
- Q: Should the library use the otel-arrow Rust crate or implement Arrow Flight independently? → A: Use the otel-arrow Rust crate as the foundation for Arrow Flight support
- Q: Which port configuration should be used for simultaneous protocol support? → A: Standard OTLP port (4317) for Protobuf, configurable port (default 4318) for Arrow Flight
- Q: Which protocols should be enabled by default when the library starts? → A: Both protocols enabled by default
- Q: Should the mock service support both gRPC protocols for testing? → A: Support both protocols - mock service accepts messages via both Protobuf and Arrow Flight gRPC
- Q: How should forwarding handle format selection and conversion? → A: Forwarding service must allow selecting output format (gRPC Protobuf or gRPC Arrow Flight) via config. If original records are in a different format, they must be converted to the selected output format
- Q: What is the minimum supported Python version? → A: Python 3.11

## Requirements *(mandatory)*

### Functional Requirements

- **FR-001**: Library MUST be usable as both a standalone service and as an embedded library with public API methods
- **FR-001a**: Public API methods MUST be callable from both Rust projects (native Rust API) and Python projects (via Python bindings/FFI, e.g., PyO3)
- **FR-001b**: Python bindings MUST support Python 3.11 or higher
- **FR-002**: Library MUST receive OTLP messages using gRPC with Protobuf protocol (standard OTLP)
- **FR-002a**: Library MUST receive OTLP messages using gRPC with Arrow Flight IPC protocol (OTAP)
- **FR-002b**: Library MUST support both gRPC protocols simultaneously on different ports/endpoints
- **FR-002c**: Library MUST provide configuration to enable/disable each protocol independently
- **FR-002g**: Library MUST enable both protocols by default (Protobuf on port 4317, Arrow Flight on port 4318)
- **FR-002d**: Library MUST use the `otel-arrow` Rust crate for Arrow Flight IPC protocol implementation
- **FR-002e**: Library MUST use standard OTLP port (4317) for gRPC with Protobuf protocol
- **FR-002f**: Library MUST use configurable port (default 4318) for gRPC with Arrow Flight IPC protocol
- **FR-003**: Library MUST write received OTLP metrics messages to local files in Arrow IPC Streaming format
- **FR-004**: Library MUST write received OTLP trace messages to local files in Arrow IPC Streaming format
- **FR-005**: Library MUST organize output files by category: metrics in {OUTPUT_DIR}/otlp/metrics and traces in {OUTPUT_DIR}/otlp/traces
- **FR-006**: Library MUST provide configuration option for output directory with default value of ./output_dir
- **FR-007**: Library MUST provide configuration option for write frequency (how often batches are written to disk) with default value of 5 seconds
- **FR-008**: Library MUST provide configuration option for trace file cleanup frequency with default value of 600 seconds
- **FR-009**: Library MUST provide configuration option for metric file cleanup frequency with default value of 1 hour (3600 seconds)
- **FR-010**: Library MUST support optional forwarding of OTLP messages to remote endpoints using either gRPC with Protobuf or gRPC with Arrow Flight IPC
- **FR-010a**: Forwarding protocol MUST be configurable per remote endpoint (Protobuf or Arrow Flight)
- **FR-010b**: When forwarding messages, if the original message format differs from the configured forwarding format, the library MUST convert messages to the selected output format (Protobuf ↔ Arrow Flight conversion)
- **FR-011**: Forwarding MUST be disabled by default
- **FR-012**: Forwarding MUST support configurable authentication according to OTLP standards
- **FR-013**: Library MUST use a unified configuration mechanism for all settings including forwarding
- **FR-014**: Library MUST be cross-platform compatible (Windows, Linux, macOS)
- **FR-015**: Library MUST preserve message ordering and data integrity during batching and writing
- **FR-016**: Library MUST handle errors gracefully without losing messages or crashing
- **FR-017**: Library MUST support concurrent message processing
- **FR-018**: Library MUST clean up old files according to configured retention intervals
- **FR-019**: Library MUST include a full mock service that can receive and process OTLP messages for testing purposes
- **FR-020**: Mock service MUST support testing via gRPC interface to validate end-to-end gRPC message flow
- **FR-021**: Mock service MUST support testing via exposed public API methods to validate end-to-end programmatic usage
- **FR-022**: Mock service MUST enable end-to-end usability testing without requiring external OTLP service dependencies
- **FR-023**: Mock service MUST support both gRPC protocols (Protobuf and Arrow Flight) for comprehensive protocol testing

### Key Entities *(include if feature involves data)*

- **OTLP Message**: Represents telemetry data (metrics or traces) received via the OpenTelemetry Protocol, contains structured data that must be preserved during processing
- **Configuration**: Represents all configurable settings including output directory, write intervals, cleanup intervals, and forwarding settings, must be validated and applied at startup
- **Output File**: Represents a file written to disk containing batched OTLP messages in Arrow IPC Streaming format, organized by category (metrics/traces) and subject to cleanup based on age
- **Remote Endpoint**: Represents a destination for forwarded messages, includes URL, protocol format (Protobuf or Arrow Flight), and authentication configuration, optional component that doesn't affect core functionality when disabled. Messages are converted to the configured format if they arrive in a different format
- **Mock Service**: Represents a testing component that simulates OTLP message generation and reception, supports both gRPC interface and public API method testing, enables end-to-end validation without external dependencies

## Success Criteria *(mandatory)*

### Measurable Outcomes

- **SC-001**: Library successfully receives and stores at least 1000 OTLP messages per second without data loss
- **SC-002**: Messages are written to disk within the configured write interval (default 5 seconds) with 100% reliability
- **SC-003**: Output files are readable and parseable as Arrow IPC Streaming format by standard Arrow libraries
- **SC-004**: Library can be integrated into another application via public API methods within 15 minutes of setup
- **SC-004a**: Public API methods can be called from Rust projects using native Rust API
- **SC-004b**: Public API methods can be called from Python projects using Python bindings (e.g., PyO3) with Python 3.11 or higher
- **SC-005**: Configuration changes take effect without requiring code modifications or recompilation
- **SC-006**: When forwarding is enabled, messages are successfully forwarded to remote endpoints with at least 99% success rate under normal network conditions, with format conversion (Protobuf ↔ Arrow Flight) performed automatically when needed
- **SC-007**: File cleanup operations remove files older than configured intervals with 100% accuracy
- **SC-008**: Library handles errors (disk full, network failures, invalid messages) without crashing or losing in-flight messages
- **SC-009**: Library operates correctly on all three target platforms (Windows, Linux, macOS) with identical behavior
- **SC-010**: Library processes messages with latency under 100ms for p95 percentile (time from message receipt to batch write initiation)
- **SC-011**: Mock service enables complete end-to-end testing of both gRPC interface and public API methods within a single test execution
- **SC-012**: End-to-end tests using the mock service complete successfully for both integration paths (gRPC and public API) with 100% test coverage of message flow
