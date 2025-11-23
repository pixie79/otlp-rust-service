# Tasks: OTLP Arrow Flight Library

**Input**: Design documents from `/specs/001-otlp-arrow-library/`
**Prerequisites**: plan.md (required), spec.md (required for user stories), research.md, data-model.md, contracts/

**Tests**: Tests are included as TDD is mandatory per constitution (II. Testing Standards). Tests must be written first, fail, then implementation follows.

**Organization**: Tasks are grouped by user story to enable independent implementation and testing of each story.

## Format: `[ID] [P?] [Story] Description`

- **[P]**: Can run in parallel (different files, no dependencies)
- **[Story]**: Which user story this task belongs to (e.g., US1, US2, US3, US4)
- Include exact file paths in descriptions

## Path Conventions

- **Single project**: `src/`, `tests/` at repository root
- Paths follow plan.md structure: `src/lib.rs`, `src/config/`, `src/otlp/`, `src/api/`, `src/mock/`, `src/bin/`

---

## Phase 1: Setup (Shared Infrastructure)

**Purpose**: Project initialization and basic structure

- [x] T001 Create project structure per implementation plan in repository root
- [x] T002 Initialize Rust library project with Cargo.toml in repository root
- [x] T003 [P] Add dependencies to Cargo.toml: opentelemetry (0.31), opentelemetry-sdk (0.31), opentelemetry-otlp (0.31), opentelemetry-proto (0.31), arrow (57), arrow-array (57), tokio (1.35+), tonic (0.14), serde (1.0), serde_yaml (0.9), anyhow (1.0), tracing (0.1)
- [x] T003a [P] Add otel-arrow dependency to Cargo.toml for Arrow Flight IPC support (commented until git dependency available)
- [x] T003b [P] Add pyo3 dependency to Cargo.toml for Python bindings support
- [x] T004 [P] Configure rustfmt.toml with project formatting standards
- [x] T005 [P] Configure clippy.toml with linting rules
- [x] T006 [P] Create .gitignore with Rust patterns and output directories
- [x] T007 [P] Create README.md with project overview and basic usage
- [x] T008 [P] Setup test directory structure: tests/unit/, tests/integration/, tests/contract/

---

## Phase 2: Foundational (Blocking Prerequisites)

**Purpose**: Core infrastructure that MUST be complete before ANY user story can be implemented

**⚠️ CRITICAL**: No user story work can begin until this phase is complete

- [x] T009 Create error types module in src/error.rs with OtlpError, OtlpConfigError, OtlpExportError, OtlpServerError
- [x] T010 [P] Create Configuration entity in src/config/types.rs with Config struct, ForwardingConfig, AuthConfig per data-model.md
- [x] T010a [P] Add ProtocolConfig struct to src/config/types.rs with protobuf_enabled, protobuf_port, arrow_flight_enabled, arrow_flight_port fields per data-model.md
- [x] T010b [P] Add ForwardingProtocol enum to src/config/types.rs with Protobuf and ArrowFlight variants per data-model.md
- [x] T011 [P] Create configuration loader in src/config/loader.rs supporting YAML, environment variables (OTLP_* prefix), and programmatic API
- [x] T011a [P] Add protocol configuration loading from YAML and environment variables in src/config/loader.rs (OTLP_PROTOBUF_ENABLED, OTLP_PROTOBUF_PORT, OTLP_ARROW_FLIGHT_ENABLED, OTLP_ARROW_FLIGHT_PORT)
- [x] T012 [P] Create configuration validation in src/config/types.rs with validation rules from data-model.md
- [x] T012a [P] Add protocol configuration validation in src/config/types.rs (at least one protocol enabled, ports different, valid port ranges)
- [x] T013 Create library root module in src/lib.rs with public API re-exports
- [x] T014 [P] Setup structured logging infrastructure using tracing in src/lib.rs
- [x] T015 Create base message types in src/otlp/mod.rs: MessageType enum, OtlpMessage struct per data-model.md

**Checkpoint**: Foundation ready - user story implementation can now begin in parallel

---

## Phase 3: User Story 1 - Core OTLP Ingestion and Local Storage (Priority: P1) 🎯 MVP

**Goal**: Receive OTLP messages via gRPC (Protobuf and Arrow Flight) and public API, buffer them, and write to local Arrow IPC files organized by type (traces/metrics)

**Independent Test**: Send OTLP messages to the library (via gRPC Protobuf, gRPC Arrow Flight, or public API) and verify that files are created in the expected directory structure ({OUTPUT_DIR}/otlp/traces and {OUTPUT_DIR}/otlp/metrics) with valid Arrow IPC Streaming format data

### Tests for User Story 1 ⚠️

> **NOTE: Write these tests FIRST, ensure they FAIL before implementation**

- [x] T016 [P] [US1] Unit test for Arrow IPC conversion in tests/unit/otlp/test_arrow_conversion.rs
- [x] T017 [P] [US1] Unit test for batch buffer in tests/unit/otlp/test_batch_buffer.rs
- [x] T018 [P] [US1] Integration test for gRPC Protobuf trace ingestion in tests/integration/test_grpc_protobuf_trace_ingestion.rs (placeholder until server conversion functions implemented)
- [x] T019 [P] [US1] Integration test for gRPC Protobuf metrics ingestion in tests/integration/test_grpc_protobuf_metrics_ingestion.rs (placeholder until server conversion functions implemented)
- [x] T020 [P] [US1] Integration test for gRPC Arrow Flight trace ingestion in tests/integration/test_grpc_arrow_flight_trace_ingestion.rs
- [x] T021 [P] [US1] Integration test for gRPC Arrow Flight metrics ingestion in tests/integration/test_grpc_arrow_flight_metrics_ingestion.rs
- [x] T022 [P] [US1] Integration test for public API trace export in tests/integration/test_public_api_traces.rs
- [x] T023 [P] [US1] Integration test for public API metrics export in tests/integration/test_public_api_metrics.rs
- [x] T024 [P] [US1] Contract test for OTLP protocol compliance in tests/contract/test_otlp_protocol.rs
- [x] T025 [P] [US1] Integration test for file writing with default output directory in tests/integration/test_file_writing.rs

### Implementation for User Story 1

- [x] T026 [P] [US1] Create BatchBuffer struct in src/otlp/batch_writer.rs with traces and metrics buffers per data-model.md
- [x] T027 [P] [US1] Implement Arrow IPC conversion for traces in src/otlp/exporter.rs using convert_spans_to_arrow_ipc function (based on cap-gl-consumer-rust patterns)
- [x] T028 [P] [US1] Implement Arrow IPC conversion for metrics in src/otlp/exporter.rs using convert_metrics_to_arrow_ipc function
- [x] T029 [US1] Create OtlpFileExporter struct in src/otlp/exporter.rs with TracesWriter and MetricsWriter (based on cap-gl-consumer-rust/src/otlp/file_exporter.rs)
- [x] T030 [US1] Implement file writing with Arrow IPC Streaming format in src/otlp/exporter.rs using arrow::ipc::writer::StreamWriter
- [x] T031 [US1] Implement file rotation logic in src/otlp/exporter.rs based on file size (max_file_size)
- [x] T032 [US1] Implement batched writing with interval timer (default 5 seconds) in src/otlp/batch_writer.rs using tokio intervals
- [x] T033 [US1] Create directory structure {OUTPUT_DIR}/otlp/traces and {OUTPUT_DIR}/otlp/metrics in src/otlp/exporter.rs
- [x] T034 [US1] Implement FileSpanExporter in src/otlp/exporter.rs implementing opentelemetry_sdk::trace::SpanExporter trait
- [x] T035 [US1] Implement FileMetricExporter in src/otlp/exporter.rs implementing opentelemetry_sdk::metrics::exporter::PushMetricExporter trait
- [x] T036 [US1] Create gRPC Protobuf server in src/otlp/server.rs using tonic with TraceService and MetricsService implementations
- [x] T037 [US1] Implement TraceService::Export in src/otlp/server.rs to receive OTLP trace messages via gRPC Protobuf
- [x] T038 [US1] Implement MetricsService::Export in src/otlp/server.rs to receive OTLP metrics messages via gRPC Protobuf
- [x] T039 [US1] Create gRPC Arrow Flight server in src/otlp/server_arrow.rs using otel-arrow crate for OTAP protocol (structure created, pending otel-arrow dependency)
- [x] T040 [US1] Implement Arrow Flight trace service in src/otlp/server_arrow.rs to receive OTLP trace messages via Arrow Flight IPC
- [x] T041 [US1] Implement Arrow Flight metrics service in src/otlp/server_arrow.rs to receive OTLP metrics messages via Arrow Flight IPC
- [x] T042 [US1] Integrate dual protocol server startup in src/bin/main.rs (start both Protobuf and Arrow Flight servers based on ProtocolConfig)
- [x] T043 [US1] Create public API module in src/api/public.rs with OtlpLibrary struct
- [x] T044 [US1] Implement OtlpLibrary::new in src/api/public.rs accepting Config parameter
- [x] T045 [US1] Implement OtlpLibrary::export_trace in src/api/public.rs for single span export
- [x] T046 [US1] Implement OtlpLibrary::export_traces in src/api/public.rs for multiple spans export
- [x] T047 [US1] Implement OtlpLibrary::export_metrics in src/api/public.rs for metrics export
- [x] T048 [US1] Implement OtlpLibrary::flush in src/api/public.rs to force immediate batch write
- [x] T049 [US1] Implement OtlpLibrary::shutdown in src/api/public.rs for graceful shutdown
- [x] T050 [US1] Create standalone service binary in src/bin/main.rs that starts gRPC server and processes messages
- [x] T051 [US1] Add error handling for file system errors, disk full, and write failures in src/otlp/exporter.rs
- [x] T052 [US1] Add structured logging for trace and metric export operations in src/otlp/exporter.rs

**Checkpoint**: At this point, User Story 1 should be fully functional and testable independently. OTLP messages can be received via gRPC Protobuf, gRPC Arrow Flight, or public API and written to Arrow IPC files.

---

## Phase 4: User Story 2 - Configuration System (Priority: P2)

**Goal**: Provide configuration mechanism for output directory, write frequency, cleanup schedules, and protocol settings without code changes

**Independent Test**: Configure the library with different settings (output directory, write interval, cleanup intervals, protocol enable/disable) and verify that all configuration options are respected and files are written/cleaned according to configuration

### Tests for User Story 2 ⚠️

- [x] T053 [P] [US2] Unit test for configuration loading from YAML in tests/unit/config/test_yaml_loader.rs
- [x] T054 [P] [US2] Unit test for configuration loading from environment variables in tests/test_env_loader.rs
- [x] T055 [P] [US2] Unit test for configuration validation in tests/test_config_validation.rs
- [x] T056 [P] [US2] Unit test for protocol configuration validation in tests/test_protocol_config.rs
- [x] T057 [P] [US2] Integration test for custom output directory in tests/integration/test_custom_output_dir.rs
- [x] T058 [P] [US2] Integration test for custom write interval in tests/integration/test_custom_write_interval.rs
- [x] T059 [P] [US2] Integration test for trace cleanup interval in tests/integration/test_trace_cleanup.rs
- [x] T060 [P] [US2] Integration test for metric cleanup interval in tests/integration/test_metric_cleanup.rs
- [x] T061 [P] [US2] Integration test for protocol enable/disable configuration in tests/integration/test_protocol_config_integration.rs
- [x] T062 [P] [US2] Integration test for configuration with defaults in tests/integration/test_config_defaults.rs

### Implementation for User Story 2

- [x] T063 [P] [US2] Implement ConfigBuilder pattern in src/config/types.rs with builder methods: output_dir, write_interval_secs, trace_cleanup_interval_secs, metric_cleanup_interval_secs, protocols
- [x] T064 [US2] Enhance configuration loader in src/config/loader.rs to support loading from YAML file with serde_yaml
- [x] T065 [US2] Enhance configuration loader in src/config/loader.rs to support loading from environment variables with OTLP_* prefix (OTLP_OUTPUT_DIR, OTLP_WRITE_INTERVAL_SECS, etc.)
- [x] T066 [US2] Implement configuration priority: provided config > environment variables > defaults in src/config/loader.rs
- [x] T067 [US2] Integrate custom output directory configuration into OtlpFileExporter in src/otlp/exporter.rs
- [x] T068 [US2] Integrate custom write interval configuration into batch writer in src/otlp/batch_writer.rs
- [x] T069 [US2] Implement file cleanup task for traces in src/otlp/exporter.rs using tokio background task with trace_cleanup_interval_secs
- [x] T070 [US2] Implement file cleanup task for metrics in src/otlp/exporter.rs using tokio background task with metric_cleanup_interval_secs
- [x] T071 [US2] Implement cleanup logic checking file modification time against cleanup interval in src/otlp/exporter.rs (based on cap-gl-consumer-rust cleanup pattern)
- [x] T072 [US2] Integrate protocol configuration into server startup in src/bin/main.rs (enable/disable Protobuf and Arrow Flight servers based on ProtocolConfig)
- [x] T073 [US2] Add configuration validation for invalid paths, negative intervals, and unreasonable values in src/config/types.rs
- [x] T074 [US2] Add structured logging for configuration loading and validation in src/config/loader.rs

**Checkpoint**: At this point, User Stories 1 AND 2 should both work independently. Library is configurable via YAML, environment variables, or programmatic API, with protocol enable/disable support.

---

## Phase 5: User Story 3 - Testing and Development Support (Priority: P3)

**Goal**: Provide mock OTLP service that supports testing via both gRPC Protobuf, gRPC Arrow Flight, and public API methods for end-to-end validation

**Independent Test**: Use the mock service to send messages via both gRPC Protobuf, gRPC Arrow Flight, and public API methods, then verify that messages are correctly processed and stored, validating all integration paths

### Tests for User Story 3 ⚠️

- [x] T075 [P] [US3] Unit test for MockOtlpService creation and state management in tests/test_mock_service.rs
- [x] T076 [P] [US3] Integration test for mock service gRPC Protobuf interface in tests/integration/test_mock_service_grpc_protobuf.rs
- [x] T077 [P] [US3] Integration test for mock service gRPC Arrow Flight interface in tests/integration/test_mock_service_grpc_arrow_flight.rs
- [x] T078 [P] [US3] Integration test for mock service public API interface in tests/integration/test_mock_service_api.rs
- [x] T079 [P] [US3] Integration test for end-to-end testing with mock service in tests/integration/test_mock_service_e2e.rs
- [x] T080 [P] [US3] Integration test for mock service message validation in tests/integration/test_mock_service_validation.rs

### Implementation for User Story 3

- [x] T081 [P] [US3] Create MockServiceState struct in src/mock/service.rs with received_traces, received_metrics, grpc_calls, api_calls per data-model.md
- [x] T082 [US3] Create MockOtlpService struct in src/mock/service.rs implementing both gRPC server and public API
- [x] T083 [US3] Implement MockOtlpService::new in src/mock/service.rs
- [x] T084 [US3] Implement MockOtlpService gRPC Protobuf server in src/mock/service.rs with TraceService and MetricsService implementations
- [x] T085 [US3] Implement MockOtlpService gRPC Arrow Flight server in src/mock/service.rs with Arrow Flight service implementations
- [x] T086 [US3] Implement MockOtlpService::start in src/mock/service.rs to start both Protobuf and Arrow Flight servers and return addresses
- [x] T087 [US3] Implement MockOtlpService::receive_trace in src/mock/service.rs for public API trace reception
- [x] T088 [US3] Implement MockOtlpService::receive_metric in src/mock/service.rs for public API metrics reception
- [x] T089 [US3] Implement MockOtlpService::assert_traces_received in src/mock/service.rs for test assertions
- [x] T090 [US3] Implement MockOtlpService::assert_metrics_received in src/mock/service.rs for test assertions
- [x] T091 [US3] Implement MockOtlpService::reset in src/mock/service.rs for test isolation
- [x] T092 [US3] Add mock service module export in src/mock/mod.rs
- [x] T093 [US3] Export MockOtlpService in public API in src/lib.rs

**Checkpoint**: At this point, User Stories 1, 2, AND 3 should all work independently. Mock service enables comprehensive end-to-end testing of both gRPC protocols and public API integration paths.

---

## Phase 6: User Story 4 - Optional Remote Forwarding (Priority: P4)

**Goal**: Support optional forwarding of OTLP messages to remote endpoints with configurable protocol (Protobuf or Arrow Flight) and automatic format conversion, while maintaining local storage as primary function

**Independent Test**: Enable forwarding configuration and verify that messages are both stored locally and forwarded to the remote endpoint in the configured format, with automatic format conversion when input format differs, and graceful handling of forwarding failures

### Tests for User Story 4 ⚠️

- [x] T094 [P] [US4] Unit test for ForwardingConfig validation in tests/unit/otlp/test_forwarding_config.rs
- [x] T095 [P] [US4] Unit test for authentication configuration in tests/unit/otlp/test_auth_config.rs
- [x] T096 [P] [US4] Unit test for format conversion Protobuf to Arrow Flight in tests/unit/otlp/test_format_conversion.rs
- [x] T097 [P] [US4] Unit test for format conversion Arrow Flight to Protobuf in tests/unit/otlp/test_format_conversion.rs
- [x] T098 [P] [US4] Integration test for forwarding disabled (default) in tests/integration/test_forwarding_disabled.rs
- [x] T099 [P] [US4] Integration test for forwarding enabled with Protobuf endpoint in tests/integration/test_forwarding_protobuf.rs
- [x] T100 [P] [US4] Integration test for forwarding enabled with Arrow Flight endpoint in tests/integration/test_forwarding_arrow_flight.rs
- [x] T101 [P] [US4] Integration test for forwarding with format conversion (Protobuf input → Arrow Flight output) in tests/integration/test_forwarding_conversion.rs
- [x] T102 [P] [US4] Integration test for forwarding with format conversion (Arrow Flight input → Protobuf output) in tests/integration/test_forwarding_conversion.rs
- [x] T103 [P] [US4] Integration test for forwarding with authentication in tests/integration/test_forwarding_auth.rs
- [x] T104 [P] [US4] Integration test for forwarding failure handling in tests/integration/test_forwarding_failure.rs
- [x] T105 [P] [US4] Integration test for local storage continuing during forwarding failures in tests/integration/test_forwarding_resilience.rs

### Implementation for User Story 4

- [x] T106 [P] [US4] Create ForwardingConfig struct in src/config/types.rs with enabled, endpoint_url, protocol, authentication fields per data-model.md
- [x] T107 [P] [US4] Create AuthConfig struct in src/config/types.rs with auth_type and credentials fields per data-model.md
- [x] T108 [US4] Create FormatConverter struct in src/otlp/converter.rs for Protobuf ↔ Arrow Flight conversion
- [x] T109 [US4] Implement FormatConverter::protobuf_to_arrow_flight in src/otlp/converter.rs using otel-arrow crate for encoding
- [x] T110 [US4] Implement FormatConverter::arrow_flight_to_protobuf in src/otlp/converter.rs using opentelemetry-otlp protobuf encoding
- [x] T111 [US4] Create OtlpForwarder struct in src/otlp/forwarder.rs for remote forwarding with format conversion support
- [x] T112 [US4] Implement OtlpForwarder::new in src/otlp/forwarder.rs accepting ForwardingConfig
- [x] T113 [US4] Implement remote forwarding using opentelemetry-otlp exporter for Protobuf in src/otlp/forwarder.rs
- [x] T114 [US4] Implement remote forwarding using otel-arrow for Arrow Flight in src/otlp/forwarder.rs
- [x] T115 [US4] Implement format conversion detection and conversion in src/otlp/forwarder.rs (check input format vs forwarding protocol, convert if needed)
- [x] T116 [US4] Implement authentication header injection in src/otlp/forwarder.rs for api_key, bearer_token, basic auth types
- [x] T117 [US4] Integrate forwarding into OtlpFileExporter in src/otlp/exporter.rs (forward after local write)
- [x] T118 [US4] Implement asynchronous forwarding to avoid blocking local storage in src/otlp/forwarder.rs
- [x] T119 [US4] Implement error handling for forwarding failures (log, don't fail) in src/otlp/forwarder.rs
- [x] T120 [US4] Implement error handling for format conversion failures (log, don't fail) in src/otlp/converter.rs
- [x] T121 [US4] Implement circuit breaker pattern for repeated forwarding failures in src/otlp/forwarder.rs
- [x] T122 [US4] Add forwarding configuration to ConfigBuilder in src/config/types.rs
- [x] T123 [US4] Add forwarding configuration validation in src/config/types.rs (endpoint_url required if enabled, valid URL format, protocol selection)
- [x] T124 [US4] Add structured logging for forwarding operations in src/otlp/forwarder.rs
- [x] T125 [US4] Add structured logging for format conversion operations in src/otlp/converter.rs

**Checkpoint**: At this point, all user stories should be independently functional. Library supports local storage with optional remote forwarding, automatic format conversion, and both Protobuf and Arrow Flight protocols.

---

## Phase 7: Python Bindings (Priority: P5)

**Goal**: Enable Python projects to call public API methods via Python bindings using PyO3 (Python 3.11+ required)

**Independent Test**: Import Python module, create OtlpLibrary instance, call export methods from Python, and verify messages are processed and stored correctly

### Tests for Python Bindings ⚠️

- [x] T126 [P] [US5] Python unit test for library initialization in tests/python/test_library_init.py
- [x] T127 [P] [US5] Python unit test for trace export in tests/python/test_trace_export.py
- [x] T128 [P] [US5] Python unit test for metrics export in tests/python/test_metrics_export.py
- [x] T129 [P] [US5] Python integration test for end-to-end usage in tests/python/test_integration.py

### Implementation for Python Bindings

- [x] T130 [P] [US5] Create Python bindings module structure in src/python/mod.rs
- [x] T131 [US5] Create PyO3 bindings in src/python/bindings.rs wrapping OtlpLibrary struct with #[pyclass]
- [x] T132 [US5] Implement Python bindings for OtlpLibrary::new in src/python/bindings.rs with #[pymethods]
- [x] T133 [US5] Implement Python bindings for OtlpLibrary::export_trace in src/python/bindings.rs
- [x] T134 [US5] Implement Python bindings for OtlpLibrary::export_traces in src/python/bindings.rs
- [x] T135 [US5] Implement Python bindings for OtlpLibrary::export_metrics in src/python/bindings.rs
- [x] T136 [US5] Implement Python bindings for OtlpLibrary::flush in src/python/bindings.rs
- [x] T137 [US5] Implement Python bindings for OtlpLibrary::shutdown in src/python/bindings.rs
- [x] T138 [US5] Implement Python error handling (convert Rust errors to Python exceptions) in src/python/bindings.rs
- [x] T139 [US5] Configure maturin build system for Python wheel generation in pyproject.toml with requires-python = ">=3.11"
- [x] T140 [US5] Add Python module export in src/lib.rs
- [x] T141 [US5] Create Python usage examples in examples/python_example.py

**Checkpoint**: At this point, Python projects can use the library via Python bindings with full API access.

---

## Phase 8: Polish & Cross-Cutting Concerns

**Purpose**: Improvements that affect multiple user stories

- [x] T142 [P] Add comprehensive documentation comments to all public APIs in src/api/public.rs
- [x] T143 [P] Add comprehensive documentation comments to all configuration types in src/config/types.rs
- [x] T144 [P] Add comprehensive documentation comments to format conversion in src/otlp/converter.rs
- [x] T145 [P] Create examples directory with usage examples: examples/standalone.rs, examples/embedded.rs, examples/python_example.py
- [x] T146 [P] Add performance benchmarks for message throughput in tests/bench/bench_throughput.rs
- [x] T147 [P] Add performance benchmarks for latency in tests/bench/bench_latency.rs
- [x] T148 [P] Add performance benchmarks for format conversion overhead in tests/bench/bench_format_conversion.rs
- [x] T149 [P] Implement health check endpoint for standalone service in src/bin/main.rs
- [x] T150 [P] Add metrics collection for library operations (messages received, files written, errors, format conversions) in src/otlp/exporter.rs
- [x] T151 [P] Add cross-platform testing validation (Windows, Linux, macOS) in CI/CD configuration
- [x] T152 [P] Run code coverage validation to ensure 85% per file requirement in CI/CD
- [x] T153 [P] Update README.md with complete usage examples and API documentation including dual protocol and format conversion
- [x] T154 [P] Create CHANGELOG.md with version history
- [x] T155 [P] Validate quickstart.md examples work correctly with dual protocol and format conversion

---

## Dependencies & Execution Order

### Phase Dependencies

- **Setup (Phase 1)**: No dependencies - can start immediately
- **Foundational (Phase 2)**: Depends on Setup completion - BLOCKS all user stories
- **User Stories (Phase 3+)**: All depend on Foundational phase completion
  - User stories can then proceed in parallel (if staffed)
  - Or sequentially in priority order (P1 → P2 → P3 → P4 → P5)
- **Polish (Final Phase)**: Depends on all desired user stories being complete

### User Story Dependencies

- **User Story 1 (P1)**: Can start after Foundational (Phase 2) - No dependencies on other stories
- **User Story 2 (P2)**: Can start after Foundational (Phase 2) - Enhances US1 but should be independently testable
- **User Story 3 (P3)**: Can start after Foundational (Phase 2) - Uses US1 components but should be independently testable
- **User Story 4 (P4)**: Can start after Foundational (Phase 2) - Uses US1 components and format conversion, should be independently testable
- **Python Bindings (P5)**: Can start after US1 - Uses public API from US1

### Within Each User Story

- Tests (if included) MUST be written and FAIL before implementation
- Configuration before exporters
- Exporters before servers/APIs
- Format conversion before forwarding
- Core implementation before integration
- Story complete before moving to next priority

### Parallel Opportunities

- All Setup tasks marked [P] can run in parallel
- All Foundational tasks marked [P] can run in parallel (within Phase 2)
- Once Foundational phase completes, all user stories can start in parallel (if team capacity allows)
- All tests for a user story marked [P] can run in parallel
- Models/entities within a story marked [P] can run in parallel
- Different user stories can be worked on in parallel by different team members
- Format conversion and forwarding can be developed in parallel (different modules)

---

## Parallel Example: User Story 1

```bash
# Launch all tests for User Story 1 together:
Task: "Unit test for Arrow IPC conversion in tests/unit/otlp/test_arrow_conversion.rs"
Task: "Unit test for batch buffer in tests/unit/otlp/test_batch_buffer.rs"
Task: "Integration test for gRPC Protobuf trace ingestion in tests/integration/test_grpc_protobuf_trace_ingestion.rs"
Task: "Integration test for gRPC Arrow Flight trace ingestion in tests/integration/test_grpc_arrow_flight_trace_ingestion.rs"

# Launch all models/entities for User Story 1 together:
Task: "Create BatchBuffer struct in src/otlp/batch_writer.rs"
Task: "Implement Arrow IPC conversion for traces in src/otlp/exporter.rs"
Task: "Implement Arrow IPC conversion for metrics in src/otlp/exporter.rs"
Task: "Create gRPC Protobuf server in src/otlp/server.rs"
Task: "Create gRPC Arrow Flight server in src/otlp/server_arrow.rs"
```

---

## Implementation Strategy

### MVP First (User Story 1 Only)

1. Complete Phase 1: Setup
2. Complete Phase 2: Foundational (CRITICAL - blocks all stories)
3. Complete Phase 3: User Story 1 (with both Protobuf and Arrow Flight servers)
4. **STOP and VALIDATE**: Test User Story 1 independently
5. Deploy/demo if ready

### Incremental Delivery

1. Complete Setup + Foundational → Foundation ready
2. Add User Story 1 (dual protocol) → Test independently → Deploy/Demo (MVP!)
3. Add User Story 2 → Test independently → Deploy/Demo
4. Add User Story 3 → Test independently → Deploy/Demo
5. Add User Story 4 (with format conversion) → Test independently → Deploy/Demo
6. Add Python Bindings → Test independently → Deploy/Demo
7. Each story adds value without breaking previous stories

### Parallel Team Strategy

With multiple developers:

1. Team completes Setup + Foundational together
2. Once Foundational is done:
   - Developer A: User Story 1 - Protobuf server
   - Developer B: User Story 1 - Arrow Flight server
   - Developer C: User Story 2 (Configuration)
   - Developer D: User Story 3 (Mock Service)
3. Next iteration:
   - Developer A: User Story 4 - Format conversion
   - Developer B: User Story 4 - Forwarding
   - Developer C: Python Bindings
   - Developer D: Polish tasks
4. Stories complete and integrate independently

---

## Notes

- [P] tasks = different files, no dependencies
- [Story] label maps task to specific user story for traceability
- Each user story should be independently completable and testable
- Verify tests fail before implementing
- Commit after each task or logical group
- Stop at any checkpoint to validate story independently
- Avoid: vague tasks, same file conflicts, cross-story dependencies that break independence
- All tasks must maintain 85% code coverage per file (constitution requirement)
- All code must pass `cargo clippy` with no warnings before merge
- Dual protocol support requires both Protobuf and Arrow Flight servers
- Format conversion is required for flexible forwarding (US4)
- Python bindings enable Python project integration (US5)
