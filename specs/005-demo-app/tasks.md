# Tasks: Demo Rust Application

**Input**: Design documents from `/specs/005-demo-app/`
**Prerequisites**: plan.md (required), spec.md (required for user stories), research.md, data-model.md, contracts/

**Tests**: Tests are included to verify demo functionality per constitution (II. Testing Standards). Integration tests verify demo runs and generates data visible in dashboard.

**Organization**: Tasks are grouped by user story to enable independent implementation and testing of each story.

## Format: `[ID] [P?] [Story] Description`

- **[P]**: Can run in parallel (different files, no dependencies)
- **[Story]**: Which user story this task belongs to (e.g., [US1], [US2], [US3])
- Include exact file paths in descriptions

## Path Conventions

- **Example file**: `examples/demo-app.rs` at repository root
- **Test files**: `tests/integration/test_demo_app.rs` and `tests/unit/examples/test_demo_app.rs`

---

## Phase 1: Setup (Project Structure)

**Purpose**: Ensure example can be added to project

- [x] T001 Verify examples directory exists at examples/ in repository root
- [x] T002 [P] Verify Cargo.toml supports examples (check for [[example]] section or add if needed)

---

## Phase 2: Foundational (No Blocking Prerequisites)

**Purpose**: This phase is minimal as demo uses existing library infrastructure

**Note**: Demo application uses existing OTLP library, so no foundational work needed. Library infrastructure (ConfigBuilder, OtlpLibrary, dashboard) already exists.

---

## Phase 3: User Story 1 - Verify Service Functionality (Priority: P1) 🎯 MVP

**Goal**: Create demo application that enables dashboard, generates mock metrics and spans, and makes data visible in dashboard to prove service is working

**Independent Test**: Run the demo application and verify that metrics and spans appear in the dashboard within 10 seconds of starting the application. Verify Arrow IPC files are created in output directory.

### Tests for User Story 1

- [ ] T003 [P] [US1] Integration test for demo app compilation in tests/integration/test_demo_app.rs - verify `cargo build --example demo-app` succeeds
- [ ] T004 [P] [US1] Integration test for demo app execution in tests/integration/test_demo_app.rs - verify demo runs without errors
- [ ] T005 [P] [US1] Integration test for dashboard startup in tests/integration/test_demo_app.rs - verify dashboard is accessible after demo starts
- [ ] T006 [P] [US1] Integration test for data generation in tests/integration/test_demo_app.rs - verify Arrow IPC files are created in output directory
- [ ] T007 [P] [US1] Integration test for dashboard data visibility in tests/integration/test_demo_app.rs - verify data appears in dashboard within write interval

### Implementation for User Story 1

- [x] T008 [US1] Create demo application file examples/demo-app.rs with module doc comment explaining purpose
- [x] T009 [US1] Add main function with #[tokio::main] attribute in examples/demo-app.rs
- [x] T010 [US1] Initialize logging using otlp_arrow_library::init_logging() in examples/demo-app.rs
- [x] T011 [US1] Create configuration with dashboard enabled using ConfigBuilder in examples/demo-app.rs (dashboard_enabled(true), output_dir, write_interval_secs)
- [x] T012 [US1] Create OtlpLibrary instance with configuration in examples/demo-app.rs using OtlpLibrary::new(config).await?
- [x] T013 [US1] Print dashboard URL to stdout in examples/demo-app.rs (e.g., "Dashboard available at http://127.0.0.1:8080")
- [x] T014 [US1] Create function to generate mock metrics in examples/demo-app.rs returning ResourceMetrics::default()
- [x] T015 [US1] Create function to generate mock spans in examples/demo-app.rs returning Vec<SpanData> with at least 10 distinct spans
- [x] T016 [US1] Export individual metrics using library.export_metrics() in examples/demo-app.rs
- [x] T017 [US1] Export individual spans using library.export_trace() in examples/demo-app.rs
- [x] T018 [US1] Export batch spans using library.export_traces() in examples/demo-app.rs
- [x] T019 [US1] Add wait for batch writing using tokio::time::sleep() in examples/demo-app.rs (wait for write_interval_secs + buffer)
- [x] T020 [US1] Flush pending data using library.flush().await? in examples/demo-app.rs
- [x] T021 [US1] Shutdown gracefully using library.shutdown().await? in examples/demo-app.rs
- [x] T022 [US1] Add error handling with proper error propagation using ? operator in examples/demo-app.rs

**Checkpoint**: Demo application runs, generates data, and data appears in dashboard. MVP complete.

---

## Phase 4: User Story 2 - Reference Implementation for SDK Usage (Priority: P2)

**Goal**: Enhance demo application with comprehensive documentation, clear code structure, and examples of all SDK usage patterns to serve as reference implementation

**Independent Test**: Examine demo application code and verify it demonstrates all key SDK usage patterns (initialization, metric creation, span creation, export, shutdown) with clear comments explaining at least 80% of SDK method calls.

### Tests for User Story 2

- [ ] T023 [P] [US2] Unit test for code documentation coverage in tests/unit/examples/test_demo_app.rs - verify comment coverage meets 80% threshold
- [ ] T024 [P] [US2] Integration test for SDK pattern demonstration in tests/integration/test_demo_app.rs - verify all patterns are present (initialization, metrics, spans, export, shutdown)

### Implementation for User Story 2

- [x] T025 [US2] Add comprehensive module-level doc comment in examples/demo-app.rs explaining demo purpose and usage
- [x] T026 [US2] Add section comments organizing code: Initialization, Metric Creation, Span Creation, Export, Shutdown in examples/demo-app.rs
- [x] T027 [US2] Add inline comments explaining ConfigBuilder usage and dashboard configuration in examples/demo-app.rs
- [x] T028 [US2] Add inline comments explaining OtlpLibrary::new() initialization pattern in examples/demo-app.rs
- [x] T029 [US2] Add inline comments explaining ResourceMetrics creation and export_metrics() usage in examples/demo-app.rs
- [x] T030 [US2] Add inline comments explaining SpanData creation with span_context, parent_span_id, span_kind in examples/demo-app.rs
- [x] T031 [US2] Add inline comments explaining export_trace() and export_traces() batch export patterns in examples/demo-app.rs
- [x] T032 [US2] Add inline comments explaining flush() and shutdown() graceful shutdown pattern in examples/demo-app.rs
- [x] T033 [US2] Enhance mock span generation to demonstrate different span kinds (Server, Client, Internal) in examples/demo-app.rs
- [x] T034 [US2] Enhance mock span generation to include realistic attributes (service.name, http.method, http.status_code) in examples/demo-app.rs
- [x] T035 [US2] Add function-level doc comments for all helper functions in examples/demo-app.rs explaining purpose, parameters, return values

**Checkpoint**: Demo application serves as complete reference implementation with comprehensive documentation.

---

## Phase 5: User Story 3 - Continuous Data Generation for Testing (Priority: P3)

**Goal**: Add continuous data generation capability to demo application for testing dashboard real-time features and data visualization

**Independent Test**: Run demo application in continuous mode and verify it generates metrics and spans at regular intervals (every 2-5 seconds) with data values changing over time to demonstrate time-series patterns.

### Tests for User Story 3

- [ ] T036 [P] [US3] Integration test for continuous generation in tests/integration/test_demo_app.rs - verify data generated at intervals
- [ ] T037 [P] [US3] Integration test for graceful shutdown on Ctrl+C in tests/integration/test_demo_app.rs - verify flush and shutdown on signal

### Implementation for User Story 3

- [x] T038 [US3] Add continuous generation mode using tokio::time::interval() in examples/demo-app.rs with configurable interval (2-5 seconds)
- [x] T039 [US3] Implement data generation loop that creates metrics and spans at each interval in examples/demo-app.rs
- [x] T040 [US3] Add counter or state to generate changing metric values over time in examples/demo-app.rs to demonstrate time-series patterns
- [x] T041 [US3] Add signal handling using tokio::signal::ctrl_c() for graceful shutdown in examples/demo-app.rs
- [x] T042 [US3] Use tokio::select! to handle both data generation loop and shutdown signal in examples/demo-app.rs
- [x] T043 [US3] Ensure flush() is called before shutdown in continuous mode in examples/demo-app.rs
- [x] T044 [US3] Add user feedback messages indicating data generation status in examples/demo-app.rs

**Checkpoint**: Demo application supports continuous generation with graceful shutdown.

---

## Phase 6: Polish & Cross-Cutting Concerns

**Purpose**: Error handling, edge cases, and final polish

- [x] T045 Handle dashboard port conflict error with clear error message in examples/demo-app.rs
- [x] T046 Handle output directory creation errors with clear error message in examples/demo-app.rs
- [x] T047 Handle dashboard startup failure gracefully (log error, continue without dashboard) in examples/demo-app.rs
- [x] T048 Add validation for dashboard static directory existence in examples/demo-app.rs
- [x] T049 Ensure all code passes cargo clippy with no warnings for examples/demo-app.rs
- [x] T050 Ensure all code passes cargo fmt for examples/demo-app.rs
- [x] T051 Verify demo compiles and runs on all platforms (Windows, Linux, macOS) - manual verification
- [x] T052 Update CHANGELOG.md with demo application addition
- [x] T053 Update README.md to mention demo-app example if not already documented

---

## Dependencies

### User Story Completion Order

1. **User Story 1 (P1)** - MVP: Must complete first. Provides core functionality (dashboard, basic metrics/spans generation, export)
2. **User Story 2 (P2)** - Reference: Can start after US1 core implementation, but documentation can be added incrementally
3. **User Story 3 (P3)** - Continuous: Requires US1 complete. Can be implemented in parallel with US2 documentation

### Task Dependencies

- T008-T022 (US1 implementation): Must complete in order (sequential dependencies)
- T025-T035 (US2 documentation): Can be done in parallel after US1 core (T008-T022) complete
- T038-T044 (US3 continuous): Requires US1 complete, can be done in parallel with US2
- T045-T053 (Polish): Requires all user stories complete

## Parallel Execution Opportunities

### Within User Story 1
- T003-T007 (Tests): Can be written in parallel
- T014-T015 (Helper functions): Can be created in parallel

### Within User Story 2
- T025-T035 (Documentation tasks): Can be done in parallel after US1 core complete

### Across Stories
- US2 documentation (T025-T035) and US3 continuous (T038-T044) can be done in parallel after US1 complete

## Implementation Strategy

### MVP Scope (User Story 1 Only)

**Minimum Viable Product**: Complete User Story 1 to deliver working demo that proves service functionality.

**MVP Tasks**: T001-T002 (Setup), T003-T007 (Tests), T008-T022 (Implementation)

**MVP Deliverable**: Demo application that:
- Enables dashboard
- Generates and exports mock metrics and spans
- Makes data visible in dashboard
- Demonstrates basic SDK usage patterns

### Incremental Delivery

1. **Phase 1**: Setup (T001-T002) - 1 task
2. **Phase 2**: Foundational - 0 tasks (uses existing infrastructure)
3. **Phase 3**: MVP (US1) - 20 tasks (T003-T022)
4. **Phase 4**: Reference (US2) - 13 tasks (T023-T035)
5. **Phase 5**: Continuous (US3) - 9 tasks (T036-T044)
6. **Phase 6**: Polish - 9 tasks (T045-T053)

**Total Tasks**: 52 tasks

## Independent Test Criteria

### User Story 1
- **Test**: Run `cargo run --example demo-app`
- **Verify**: 
  - Demo compiles and runs without errors
  - Dashboard is accessible at http://127.0.0.1:8080
  - Metrics and spans appear in dashboard within 10 seconds
  - Arrow IPC files exist in `./output_dir/otlp/` directories

### User Story 2
- **Test**: Examine `examples/demo-app.rs` code
- **Verify**:
  - All SDK usage patterns demonstrated (initialization, metrics, spans, export, shutdown)
  - At least 80% of SDK method calls have comments
  - Code structure is clear and organized
  - Documentation explains purpose and usage

### User Story 3
- **Test**: Run demo in continuous mode, let it run for 30 seconds, then Ctrl+C
- **Verify**:
  - Data generated at regular intervals (every 2-5 seconds)
  - Metric values change over time (time-series pattern visible)
  - Graceful shutdown on Ctrl+C (flush and shutdown complete)
  - No errors or panics during execution

