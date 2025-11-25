# Tasks: Python OpenTelemetry SDK Adapter Classes

**Input**: Design documents from `/specs/004-python-otel-adapters/`
**Prerequisites**: plan.md (required), spec.md (required for user stories), research.md, data-model.md, contracts/

**Tests**: TDD approach - tests written first, then implementation. All tests must be written and fail before implementation begins.

**Organization**: Tasks are grouped by user story to enable independent implementation and testing of each story.

## Format: `[ID] [P?] [Story] Description`

- **[P]**: Can run in parallel (different files, no dependencies)
- **[Story]**: Which user story this task belongs to (e.g., US1, US2, US3)
- Include exact file paths in descriptions

## Path Conventions

- **Single project**: `src/`, `tests/` at repository root
- Paths follow existing project structure: `src/python/` for Python bindings

## Phase 1: Setup (Shared Infrastructure)

**Purpose**: Project initialization and basic structure

- [ ] T001 Create adapter module structure in src/python/adapters.rs
- [ ] T002 [P] Create type conversion module in src/python/adapters/conversion.rs
- [ ] T003 [P] Update src/python/mod.rs to export adapter modules
- [ ] T004 [P] Add Python OpenTelemetry SDK test dependencies to Cargo.toml (if needed for testing)

---

## Phase 2: Foundational (Blocking Prerequisites)

**Purpose**: Core infrastructure that MUST be complete before ANY user story can be implemented

**⚠️ CRITICAL**: No user story work can begin until this phase is complete

- [ ] T005 Implement error conversion utilities in src/python/adapters/conversion.rs (Rust errors → Python exceptions)
- [ ] T006 [P] Create base type conversion trait/interface in src/python/adapters/conversion.rs
- [ ] T007 [P] Implement Python garbage collection handling utilities in src/python/adapters.rs (Py<PyOtlpLibrary> reference management)

**Checkpoint**: Foundation ready - user story implementation can now begin in parallel

---

## Phase 3: User Story 1 - Direct Integration with Python OpenTelemetry SDK Metrics (Priority: P1) 🎯 MVP

**Goal**: Provide Python metric exporter adapter that implements Python OpenTelemetry SDK's `MetricExporter` interface, enabling direct use with `PeriodicExportingMetricReader` without custom adapter code.

**Independent Test**: Create a metric exporter adapter from an `OtlpLibrary` instance and use it directly with Python OpenTelemetry SDK's `PeriodicExportingMetricReader` without any custom wrapper code. Verify metrics are automatically exported to the library's storage system.

### Tests for User Story 1 ⚠️

> **NOTE: Write these tests FIRST, ensure they FAIL before implementation**

- [ ] T008 [P] [US1] Write unit test for metric type conversion in tests/unit/python/test_adapters.rs (test_convert_metric_export_result_to_dict)
- [ ] T009 [P] [US1] Write unit test for metric adapter creation in tests/unit/python/test_adapters.rs (test_metric_exporter_adapter_creation)
- [ ] T010 [P] [US1] Write Python test for metric adapter interface in tests/python/test_adapters_metrics.py (test_metric_exporter_interface)
- [ ] T011 [P] [US1] Write Python integration test for PeriodicExportingMetricReader in tests/python/test_integration_otel_sdk.py (test_metric_reader_integration)
- [ ] T012 [P] [US1] Write contract test for metric exporter adapter in tests/integration/test_python_otel_adapters.rs (test_metric_exporter_contract)

### Implementation for User Story 1

- [ ] T013 [P] [US1] Implement metric type conversion function in src/python/adapters/conversion.rs (convert_metric_export_result_to_dict)
- [ ] T014 [US1] Create PyOtlpMetricExporterAdapter struct in src/python/adapters.rs (wraps Py<PyOtlpLibrary>)
- [ ] T015 [US1] Implement export method for PyOtlpMetricExporterAdapter in src/python/adapters.rs (delegates to library.export_metrics_ref)
- [ ] T016 [US1] Implement shutdown method for PyOtlpMetricExporterAdapter in src/python/adapters.rs (no-op)
- [ ] T017 [US1] Implement force_flush method for PyOtlpMetricExporterAdapter in src/python/adapters.rs (delegates to library.flush)
- [ ] T018 [US1] Implement temporality method for PyOtlpMetricExporterAdapter in src/python/adapters.rs (returns CUMULATIVE)
- [ ] T019 [US1] Add metric_exporter method to PyOtlpLibrary in src/python/bindings.rs (returns PyOtlpMetricExporterAdapter)
- [ ] T020 [US1] Register PyOtlpMetricExporterAdapter in Python module in src/python/bindings.rs
- [ ] T021 [US1] Add error handling and logging for metric adapter operations in src/python/adapters.rs
- [ ] T022 [US1] Add validation for metric adapter lifecycle in src/python/adapters.rs

**Checkpoint**: At this point, User Story 1 should be fully functional and testable independently. Metric exporter adapter can be used with Python OpenTelemetry SDK's PeriodicExportingMetricReader.

---

## Phase 4: User Story 2 - Direct Integration with Python OpenTelemetry SDK Traces (Priority: P2)

**Goal**: Provide Python span exporter adapter that implements Python OpenTelemetry SDK's `SpanExporter` interface, enabling direct use with `BatchSpanProcessor` and `TracerProvider` without custom adapter code.

**Independent Test**: Create a span exporter adapter from an `OtlpLibrary` instance and use it directly with Python OpenTelemetry SDK's `BatchSpanProcessor` and `TracerProvider` without any custom wrapper code. Verify spans are automatically exported to the library's storage system.

### Tests for User Story 2 ⚠️

> **NOTE: Write these tests FIRST, ensure they FAIL before implementation**

- [ ] T023 [P] [US2] Write unit test for span type conversion in tests/unit/python/test_adapters.rs (test_convert_readable_span_to_dict)
- [ ] T024 [P] [US2] Write unit test for span adapter creation in tests/unit/python/test_adapters.rs (test_span_exporter_adapter_creation)
- [ ] T025 [P] [US2] Write Python test for span adapter interface in tests/python/test_adapters_spans.py (test_span_exporter_interface)
- [ ] T026 [P] [US2] Write Python integration test for BatchSpanProcessor in tests/python/test_integration_otel_sdk.py (test_span_processor_integration)
- [ ] T027 [P] [US2] Write contract test for span exporter adapter in tests/integration/test_python_otel_adapters.rs (test_span_exporter_contract)

### Implementation for User Story 2

- [ ] T028 [P] [US2] Implement span type conversion function in src/python/adapters/conversion.rs (convert_readable_span_to_dict, convert_span_sequence_to_dict_list)
- [ ] T029 [US2] Create PyOtlpSpanExporterAdapter struct in src/python/adapters.rs (wraps Py<PyOtlpLibrary>)
- [ ] T030 [US2] Implement export method for PyOtlpSpanExporterAdapter in src/python/adapters.rs (delegates to library.export_traces)
- [ ] T031 [US2] Implement shutdown method for PyOtlpSpanExporterAdapter in src/python/adapters.rs (no-op)
- [ ] T032 [US2] Implement force_flush method for PyOtlpSpanExporterAdapter in src/python/adapters.rs (delegates to library.flush)
- [ ] T033 [US2] Add span_exporter method to PyOtlpLibrary in src/python/bindings.rs (returns PyOtlpSpanExporterAdapter)
- [ ] T034 [US2] Register PyOtlpSpanExporterAdapter in Python module in src/python/bindings.rs
- [ ] T035 [US2] Add error handling and logging for span adapter operations in src/python/adapters.rs
- [ ] T036 [US2] Add validation for span adapter lifecycle in src/python/adapters.rs

**Checkpoint**: At this point, User Stories 1 AND 2 should both work independently. Both metric and span exporter adapters can be used with Python OpenTelemetry SDK.

---

## Phase 5: User Story 3 - Cross-Platform Compatibility and Python Version Support (Priority: P3)

**Goal**: Ensure adapter classes function correctly on Windows, Linux, and macOS, and support Python 3.11 or higher without platform-specific or version-specific issues.

**Independent Test**: Create and use adapter classes on different operating systems (Windows, Linux, macOS) and different Python versions (3.11+), verifying that they function correctly and produce identical behavior across all tested environments.

### Tests for User Story 3 ⚠️

> **NOTE: Write these tests FIRST, ensure they FAIL before implementation**

- [ ] T037 [P] [US3] Write cross-platform test for metric adapter in tests/python/test_adapters_metrics.py (test_metric_adapter_cross_platform)
- [ ] T038 [P] [US3] Write cross-platform test for span adapter in tests/python/test_adapters_spans.py (test_span_adapter_cross_platform)
- [ ] T039 [P] [US3] Write Python version compatibility test in tests/python/test_integration_otel_sdk.py (test_python_version_compatibility)
- [ ] T040 [P] [US3] Add CI/CD test matrix for Windows, Linux, macOS in .github/workflows/ (if not already present)

### Implementation for User Story 3

- [ ] T041 [US3] Verify type conversion works identically across platforms in src/python/adapters/conversion.rs (no platform-specific code needed, but verify)
- [ ] T042 [US3] Add platform-specific error handling if needed in src/python/adapters.rs (should not be needed, but verify)
- [ ] T043 [US3] Verify Python garbage collection handling works on all platforms in src/python/adapters.rs
- [ ] T044 [US3] Add documentation for platform compatibility in README.md or quickstart.md
- [ ] T045 [US3] Test adapters on Windows, Linux, macOS manually or via CI/CD

**Checkpoint**: All user stories should now be independently functional. Adapters work correctly on all target platforms and Python versions.

---

## Phase 6: Polish & Cross-Cutting Concerns

**Purpose**: Improvements that affect multiple user stories

- [ ] T046 [P] Update CHANGELOG.md with adapter feature additions
- [ ] T047 [P] Update README.md with adapter usage examples
- [ ] T048 [P] Verify quickstart.md examples work correctly (run quickstart validation)
- [ ] T049 [P] Add comprehensive doc comments to all adapter methods in src/python/adapters.rs
- [ ] T050 [P] Add comprehensive doc comments to type conversion functions in src/python/adapters/conversion.rs
- [ ] T051 [P] Run cargo clippy and fix all warnings in src/python/adapters.rs
- [ ] T052 [P] Run cargo fmt on all adapter code
- [ ] T053 [P] Verify code coverage meets 85% threshold for adapter modules
- [ ] T054 [P] Add performance benchmarks for type conversion in tests/bench/ (if needed)
- [ ] T055 [P] Add concurrent usage tests in tests/integration/test_python_otel_adapters.rs (test_concurrent_metric_export, test_concurrent_span_export)
- [ ] T056 [P] Add edge case tests for error scenarios in tests/python/test_adapters_metrics.py and test_adapters_spans.py
- [ ] T057 [P] Add edge case tests for lifecycle scenarios (library shutdown, adapter after shutdown) in tests/python/test_integration_otel_sdk.py
- [ ] T058 [P] Verify all tests pass (cargo test --all-features --workspace and pytest tests/python/)
- [ ] T059 [P] Update API documentation in specs/004-python-otel-adapters/contracts/python-api.md if needed
- [ ] T060 [P] Final code review and refactoring if needed

---

## Dependencies & Execution Order

### Phase Dependencies

- **Setup (Phase 1)**: No dependencies - can start immediately
- **Foundational (Phase 2)**: Depends on Setup completion - BLOCKS all user stories
- **User Stories (Phase 3+)**: All depend on Foundational phase completion
  - User stories can then proceed in parallel (if staffed)
  - Or sequentially in priority order (P1 → P2 → P3)
- **Polish (Final Phase)**: Depends on all desired user stories being complete

### User Story Dependencies

- **User Story 1 (P1)**: Can start after Foundational (Phase 2) - No dependencies on other stories
- **User Story 2 (P2)**: Can start after Foundational (Phase 2) - May share type conversion utilities with US1 but should be independently testable
- **User Story 3 (P3)**: Can start after Foundational (Phase 2) - Depends on US1 and US2 for testing, but implementation is mostly validation

### Within Each User Story

- Tests (if included) MUST be written and FAIL before implementation
- Type conversion functions before adapter implementations
- Adapter structs before adapter methods
- Core implementation before integration
- Story complete before moving to next priority

### Parallel Opportunities

- All Setup tasks marked [P] can run in parallel
- All Foundational tasks marked [P] can run in parallel (within Phase 2)
- Once Foundational phase completes, User Stories 1 and 2 can start in parallel (if team capacity allows)
- All tests for a user story marked [P] can run in parallel
- Type conversion functions within a story marked [P] can run in parallel
- Different user stories can be worked on in parallel by different team members

---

## Parallel Example: User Story 1

```bash
# Launch all tests for User Story 1 together:
Task: "Write unit test for metric type conversion in tests/unit/python/test_adapters.rs"
Task: "Write unit test for metric adapter creation in tests/unit/python/test_adapters.rs"
Task: "Write Python test for metric adapter interface in tests/python/test_adapters_metrics.py"
Task: "Write Python integration test for PeriodicExportingMetricReader in tests/python/test_integration_otel_sdk.py"
Task: "Write contract test for metric exporter adapter in tests/integration/test_python_otel_adapters.rs"

# Launch type conversion implementation:
Task: "Implement metric type conversion function in src/python/adapters/conversion.rs"
```

---

## Parallel Example: User Story 2

```bash
# Launch all tests for User Story 2 together:
Task: "Write unit test for span type conversion in tests/unit/python/test_adapters.rs"
Task: "Write unit test for span adapter creation in tests/unit/python/test_adapters.rs"
Task: "Write Python test for span adapter interface in tests/python/test_adapters_spans.py"
Task: "Write Python integration test for BatchSpanProcessor in tests/python/test_integration_otel_sdk.py"
Task: "Write contract test for span exporter adapter in tests/integration/test_python_otel_adapters.rs"

# Launch type conversion implementation:
Task: "Implement span type conversion function in src/python/adapters/conversion.rs"
```

---

## Implementation Strategy

### MVP First (User Story 1 Only)

1. Complete Phase 1: Setup
2. Complete Phase 2: Foundational (CRITICAL - blocks all stories)
3. Complete Phase 3: User Story 1 (write tests first, then implementation)
4. **STOP and VALIDATE**: Test User Story 1 independently
5. Deploy/demo if ready

### Incremental Delivery

1. Complete Setup + Foundational → Foundation ready
2. Add User Story 1 → Test independently → Deploy/Demo (MVP!)
3. Add User Story 2 → Test independently → Deploy/Demo
4. Add User Story 3 → Test independently → Deploy/Demo
5. Each story adds value without breaking previous stories

### Parallel Team Strategy

With multiple developers:

1. Team completes Setup + Foundational together
2. Once Foundational is done:
   - Developer A: User Story 1 (metrics)
   - Developer B: User Story 2 (spans) - can start in parallel with US1
   - Developer C: User Story 3 (cross-platform) - can start after US1 and US2 are complete
3. Stories complete and integrate independently

---

## Task Summary

- **Total Tasks**: 60
- **Phase 1 (Setup)**: 4 tasks
- **Phase 2 (Foundational)**: 3 tasks
- **Phase 3 (User Story 1 - Metrics)**: 15 tasks (5 tests + 10 implementation)
- **Phase 4 (User Story 2 - Spans)**: 14 tasks (5 tests + 9 implementation)
- **Phase 5 (User Story 3 - Cross-Platform)**: 9 tasks (4 tests + 5 implementation)
- **Phase 6 (Polish)**: 15 tasks

### Task Count per User Story

- **User Story 1 (P1 - Metrics)**: 15 tasks
- **User Story 2 (P2 - Spans)**: 14 tasks
- **User Story 3 (P3 - Cross-Platform)**: 9 tasks

### Parallel Opportunities Identified

- **Phase 1**: 3 parallel tasks (T002, T003, T004)
- **Phase 2**: 2 parallel tasks (T006, T007)
- **Phase 3**: 5 parallel test tasks, 1 parallel implementation task
- **Phase 4**: 5 parallel test tasks, 1 parallel implementation task
- **Phase 5**: 4 parallel test tasks
- **Phase 6**: 15 parallel tasks (all can run in parallel)

### Independent Test Criteria

- **User Story 1**: Create metric exporter adapter, use with PeriodicExportingMetricReader, verify metrics exported to library storage
- **User Story 2**: Create span exporter adapter, use with BatchSpanProcessor, verify spans exported to library storage
- **User Story 3**: Create adapters on Windows/Linux/macOS with Python 3.11+, verify identical behavior across platforms

### Suggested MVP Scope

**MVP = User Story 1 Only** (15 tasks)
- Provides core value: Python OpenTelemetry SDK metrics integration
- Independently testable and deployable
- Enables Python developers to integrate metrics without custom code
- Can be extended with User Story 2 (spans) and User Story 3 (cross-platform validation) in subsequent releases

---

## Notes

- [P] tasks = different files, no dependencies
- [Story] label maps task to specific user story for traceability
- Each user story should be independently completable and testable
- Verify tests fail before implementing
- Commit after each task or logical group
- Stop at any checkpoint to validate story independently
- Avoid: vague tasks, same file conflicts, cross-story dependencies that break independence
- TDD approach: All test tasks must be completed and verified to fail before implementation tasks begin
- Type conversion is shared between US1 and US2, but each story has its own conversion functions for independence

